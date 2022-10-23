mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{thread, path::{Path, PathBuf}};
use std::sync::{Mutex, Arc};

use flights_parser::{FlightsParser};
use airports::{KdTreeAirportFinder, Airport, AirportFinder, HashAirportFinder};

use clap::Parser;
use std::time::Instant;

type UsedAirportFinder = HashAirportFinder;

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
    let airport_finder = Arc::new(create_kd_tree(&airports));
    let flight_graph = process_flights(bin_files, airport_finder, airports.len());
    let topmost_airports = cluster(flight_graph, airports.len());

    println!("{:?}", topmost_airports);
    let _ = plot::plot_map(&airports);
}

fn create_kd_tree(airports: &Vec<Airport>) -> UsedAirportFinder {
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
    let flight_chunk = FlightsParser::parse(bin_path);
    println!("Parsing binary file... OK. Time: {:?}", start.elapsed());

    println!("Locating airports...");
    let start = Instant::now();
    dissimilarity_graph.add_flights(&flight_chunk, airport_finder);
    println!("Locating airports... OK. Time: {:?}", start.elapsed());
    dissimilarity_graph
}