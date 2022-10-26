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

// How many threads to use
const THREADS: usize = 16;


#[derive(Parser, Debug)]
#[command(
    author = "Kåre von Geier and Samuel Selleck",
    version = "1.0",
    about = "HPDSO22 Challenge Atempt",
    long_about = None
)]
struct Args {
   paths: Vec<String>
}

fn main() {

    let args = Args::parse();
    let dir_paths: Vec<_> = args.paths.iter().map(|p| PathBuf::from(p)).collect();
    let bin_files = find_bin_files(&dir_paths);

    let airports = airports::from_csv("data/airports.csv");
    let airport_finder = Arc::new(create_airport_finder(&airports));
    let flight_graph = process_flights(bin_files, airport_finder, airports.len());
    let topmost_airports = cluster(flight_graph, &airports);

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

fn find_bin_files(data_path: &Vec<PathBuf>) -> Vec<PathBuf> {
    // TODO: be able to take several folders as specified in challenge.
    // TODO: re-write as functional!
    let mut bins = vec![];

    let all_files = data_path
        .iter()
        .flat_map(|p| p.read_dir())
        .flat_map(|d| d);

    for entry in all_files  {
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

    let nbr_threads = std::cmp::min(THREADS, bin_files.len());
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

fn cluster(flight_graph: Vec<f32>, airports: &Vec<Airport>) -> Vec<String>{
    println!("Clustering...");
    let start = Instant::now();
    let topmost = clusterer::cluster(flight_graph, airports.len(), 5);
    println!("Clustering... OK. Time: {:?}", start.elapsed());

    // TODO: Get real string names
    topmost.iter()
        .map(|&ind| format!("{}, {}", airports[ind].name, airports[ind].abr))
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

    let airports = airports::from_csv("data/airports.csv");

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

    // Assert all finders give the same results
    for flight in flights.iter() {
        let start_hash = airport_finder_hash.closest_ind(flight.from_lat, flight.from_long);
        let end_hash = airport_finder_hash.closest_ind(flight.to_lat, flight.to_long);

        let start_tree = airport_finder_tree.closest_ind(flight.from_lat, flight.from_long);
        let end_tree = airport_finder_tree.closest_ind(flight.to_lat, flight.to_long);

        let start_loop = airport_finder_loop.closest_ind(flight.from_lat, flight.from_long);
        let end_loop = airport_finder_loop.closest_ind(flight.to_lat, flight.to_long);

        assert_eq!(start_hash, start_loop);
        assert_eq!(start_hash, start_tree);
        assert_eq!(end_hash, end_loop);
        assert_eq!(end_hash, end_tree);
    }

    // Get the upper triangular flight graph
    let mut dissimilarity_graph = network::FlightCountNetwork::new(airports.len());
    dissimilarity_graph.add_flights(&flights, airport_finder);
    let original_upper_triangular = dissimilarity_graph.to_u32_vec();

    // Now get it by not doing upper triangular version
    // Initialize
    let mut simple_matrix = vec![];
    for _ in 0..airports.len() {
        simple_matrix.push(vec![0; airports.len()]);
    }
    // Add all flights symetrically
    for flight in flights.iter() {
        let start = airport_finder.closest_ind(flight.from_lat, flight.from_long);
        let end = airport_finder.closest_ind(flight.to_lat, flight.to_long);
        if start == end {
            // Mirror functionality in network
            continue;
        }

        simple_matrix[start][end] += 1;
        simple_matrix[end][start] += 1;
        if start == end {
            println!("From and to the same place!? {} & {}", start, end);
            println!("Lat and long for point: {} {} to {} {}", flight.from_lat, flight.from_long, flight.to_lat, flight.to_long);
        }
    }
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

    // They should be equal!
    assert_eq!(original_upper_triangular.iter().sum::<u32>(), simple_upper_triangular.iter().sum::<u32>());

    let mut sparse_original = vec![];
    let mut sparse_simple = vec![];

    for (ind, &val) in original_upper_triangular.iter().enumerate() {
        if val != 0 {
            sparse_original.push((ind, val));
        }
    }
    for (ind, &val) in simple_upper_triangular.iter().enumerate() {
        if val != 0 {
            sparse_simple.push((ind, val));
        }
    }

    for (&v1, &v2) in sparse_original.iter().zip(sparse_simple.iter()) {
        println!("Sparse: {:?} and {:?}", v1, v2);
        assert_eq!(v1, v2);
    }

    assert_eq!(sparse_original, sparse_simple);
    assert_eq!(original_upper_triangular, simple_upper_triangular);

}
