mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{thread, path::{Path, PathBuf}};
use std::sync::{Mutex, Arc};

use flights_parser::FlightsParser;
use airports::{Airport, HashAirportFinder};

use clap::Parser;
use std::time::Instant;

type UsedAirportFinder = HashAirportFinder;

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

fn find_bin_files(data_paths: &Vec<PathBuf>) -> Vec<PathBuf> {
    // Returns vec of all files with .bin extension in the given folders

    data_paths.iter()
        .flat_map(|dir| dir.read_dir())
        .flatten()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            if let Some(ext) = path.extension() {
                ext == "bin"
            } else {
                false
            }
        })
        .collect()
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

    // Each thread is given a couple paths to binary files with flight data. It then reads them one
    // by one and creates a matrix for all of the collective flights. Then it takes a mutex and
    // updates the shared network with the flights the thread itself has found.
    for thread_bin_files in bin_file_chunks {
        let combiner_clone = combiner.clone();
        let finder_clone = airport_finder.clone();
        let running_thread =
            thread::spawn(move || {
                let mut file_graph = network::FlightCountNetwork::new(nbr_airports);
                // Combine all files' graphs into one graph
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
