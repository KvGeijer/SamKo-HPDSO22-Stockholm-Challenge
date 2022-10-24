mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{thread, path::{Path, PathBuf}};
use std::sync::{Mutex, Arc};

use flights_parser::{FlightsParser, Flight};
use airports::{KdTreeAirportFinder, Airport, AirportFinder, HashAirportFinder, DoubleLoopAirportFinder};

use clap::Parser;
use std::time::Instant;

type UsedAirportFinder = KdTreeAirportFinder;

// How many threads to use. Could make it possible to change through command line
macro_rules! THREADS {
    () => {
        16
    };
}

#[derive(Parser, Debug)]
#[command(
    author = "Kåre von Geier and Samuel Selleck",
    version = "1.0",
    about = "HPDSO22 Challenge Atempt",
    long_about = None
)]
struct Args {
   #[arg(short)]
   path: String
}

fn main() {

    let args = Args::parse();
    let data_path = Path::new(&args.path);
    let bin_files = find_bin_files(&data_path);

    let airports = airports::from_csv(&data_path.join("airports.csv"));
    let airport_finder = Arc::new(create_airport_finder(&airports));
    let flight_graph = process_flights(bin_files, airport_finder, airports.len());
    let topmost_airports = cluster(flight_graph, airports.len());

    println!("{:?}", topmost_airports);
    let _ = plot::plot_map(&airports);
}

fn create_airport_finder(airports: &Vec<Airport>) -> UsedAirportFinder {
    println!("Loading airports..");
    let start = Instant::now();
    let kdtree = UsedAirportFinder::new(airports);
    println!("Loading airport... OK. Time: {:?}", start.elapsed());
    kdtree
}

fn find_bin_files(data_path: &Path) -> Vec<PathBuf> {
    // TODO: be able to take several folders as specified in challenge.
    // TODO: re-write as functional!
    let mut bins = vec![];

    for entry in data_path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if let Some(ext) = path.extension() {
            if ext == "bin" {
                bins.push(path);
            }
        }
    }

    bins
}

fn process_flights(bin_files: Vec<PathBuf>, airport_finder: Arc<UsedAirportFinder>,
                   nbr_airports: usize) -> Vec<f32> {
    let start = Instant::now();
    println!("Parsing and adding Networks...");

    let nbr_threads = std::cmp::min(THREADS!(), bin_files.len());
    let files_per_thread = (bin_files.len() + nbr_threads - 1)/nbr_threads;
    let bin_file_chunks = bin_files
        .chunks(files_per_thread)
        .map(|chunk_slice| chunk_slice.to_vec());   // Convert slice to vec so thay are owned

    let mut running_threads = Vec::with_capacity(nbr_threads);

    let combiner = Arc::new(Mutex::new(network::FlightCountNetwork::new(nbr_airports)));

    // Iterate over binary files and convert them into matrices
    // Multithreaded: Each threads adds into the combiner network with Mutex
    for thread_bin_files in bin_file_chunks {
        let combiner_clone = combiner.clone();
        let finder_clone = airport_finder.clone();
        let running_thread =
            // TODO: Factor out thread run function!
            thread::spawn(move || {
                let mut file_graph = network::FlightCountNetwork::new(nbr_airports);
                // Combine all your files into one graph
                for bin_path in thread_bin_files {
                    file_graph = dissimilarity_from_binary(file_graph,
                                                           &bin_path.as_path(),
                                                           &finder_clone);
                }
                // Add your total graph to the collective one
                combiner_clone.lock()
                    .unwrap()
                    .add_network(file_graph);
            });
        running_threads.push(running_thread);
    }

    for thread in running_threads {
        thread.join().unwrap();
    }
    println!("Parsing and adding Networks... OK. Time: {:?}", start.elapsed());


    println!("Converting...");
    let start = Instant::now();
    let matrix = combiner.lock()
        .unwrap()
        .to_float_vec();
    println!("Converting... OK. Time: {:?}", start.elapsed());

    matrix
}

fn cluster(flight_graph: Vec<f32>, nbr_airports: usize) -> Vec<String>{
    println!("Clustering...");
    let start = Instant::now();
    let topmost = clusterer::cluster(flight_graph, nbr_airports, 5);
    println!("Clustering... OK. Time: {:?}", start.elapsed());

    // TODO: Get real string names
    topmost.iter()
        .map(|ind| ind.to_string())
        .collect()

}

fn dissimilarity_from_binary(mut dissimilarity_graph: network::FlightCountNetwork, bin_path: &Path,
                             airport_finder: &UsedAirportFinder) -> network::FlightCountNetwork {

    println!("Parsing binary file...");
    let start = Instant::now();
    let flights = FlightsParser::parse(bin_path);
    println!("Parsing binary file... OK. Time: {:?}", start.elapsed());

    println!("Locating airports...");
    let start = Instant::now();
    dissimilarity_graph.add_flights(&flights, airport_finder);
    println!("Locating airports... OK. Time: {:?}", start.elapsed());
    dissimilarity_graph
}

#[test]
fn test_airport_finding() {
    // Compare two methods of testing

    let airports = airports::from_csv(&Path::new("data/airports.csv"));

    // Compare different implementations
    let airport_finder_tree = KdTreeAirportFinder::new(&airports);
    let airport_finder_hash = HashAirportFinder::new(&airports);
    let airport_finder_loop = DoubleLoopAirportFinder::new(&airports);
    let airport_finder = &airport_finder_hash;

    // Only take a 1000 since loop is so incredibly slow
    let flights:Vec<Flight> = FlightsParser::parse(&Path::new("data/dat.bin"))
        .into_iter()
        .take(1000)
        .collect();


    for flight in flights.iter() {
        // let start_hash = airport_finder_hash.closest_ind(flight.from_lat, flight.from_long);
        // let end_has = airport_finder_hash.closest_ind(flight.to_lat, flight.to_long);

        // let start_tree = airport_finder_tree.closest_ind(flight.from_lat, flight.from_long);
        // let end_tree = airport_finder_tree.closest_ind(flight.to_lat, flight.to_long);

        // let start_loop = airport_finder_loop.closest_ind(flight.from_lat, flight.from_long);
        // let end_loop = airport_finder_loop.closest_ind(flight.to_lat, flight.to_long);

        // assert_eq!(start_hash, start_loop, start_tree);
        // assert_eq!()
    }

    // Get the upper triangular flight graph
    let mut dissimilarity_graph = network::FlightCountNetwork::new(airports.len());
    dissimilarity_graph.add_flights(&flights, airport_finder);
    let original_upper_triangular = dissimilarity_graph.to_u32_vec();

    // Should have added as many as we have flights
    assert_eq!(flights.len() as u32, original_upper_triangular.iter().sum());

    // Now get it by not doing upper triangular version
    // Initialize
    let mut simple_matrix = vec![];
    for _ in 0..airports.len() {
        simple_matrix.push(vec![0; airports.len()]);
    }
    // Add all flights symetrically
    let mut nbr_invalids = 0;
    for flight in flights.iter() {
        let start = airport_finder.closest_ind(flight.from_lat, flight.from_long);
        let end = airport_finder.closest_ind(flight.to_lat, flight.to_long);
        simple_matrix[start][end] += 1;
        simple_matrix[end][start] += 1;
        if start == end {
            nbr_invalids += 1;
            println!("From and to the same place!? {} & {}", start, end);
            println!("Lat and long for point: {} {} to {} {}", flight.from_lat, flight.from_long, flight.to_lat, flight.to_long);
        }
    }
    assert_eq!(nbr_invalids, 0);
    // Convert to upper triangular represented as vector
    let mut simple_upper_triangular = vec![0; original_upper_triangular.len()];
    let mut ind = 0;
    for row in 0..airports.len() {
        for col in (row + 1)..airports.len() {
            simple_upper_triangular[ind] = simple_matrix[row][col];
            ind += 1;
        }
    }
    assert_eq!(original_upper_triangular.len(), ind);

    // Should have added as many as we have flights
    // Does not work as we add a bunch to the diagonal...
    //assert_eq!(flights.len() as u32, simple_upper_triangular.iter().sum());

    // They should be equal!
    assert_eq!(original_upper_triangular.iter().sum::<u32>(), simple_upper_triangular.iter().sum::<u32>());
    assert_eq!(original_upper_triangular, simple_upper_triangular);

}
