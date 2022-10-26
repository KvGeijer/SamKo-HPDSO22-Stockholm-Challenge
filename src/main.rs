mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{thread, path::PathBuf};
use std::sync::{Mutex, Arc};
use clap::Parser;
use std::time::Instant;

use flights_parser::FlightsParser;
use airports::{Airport, HashAirportFinder};


// To easily toggle between different implementations
type UsedAirportFinder = HashAirportFinder;

const THREADS: usize = 32;
const NBR_DEND_TOP: usize = 5;
const AIRPORTS_PATH: &str = "data/airports.csv";


#[derive(Parser, Debug)]
#[command(
    author = "Kåre von Geier and Samuel Selleck",
    version = "1.0",
    about = "HPDSO22 Challenge Attempt",
    long_about = None
)]
struct Args {
   paths: Vec<String>
}

fn main() {
    let args = Args::parse();
    let bin_files = find_bin_files(args.paths);

    let start = Instant::now();

    let airports = airports::from_csv(AIRPORTS_PATH);
    let airport_finder = Arc::new(UsedAirportFinder::new(&airports));
    let flight_graph = process_flights(bin_files, airport_finder, airports.len());
    let topmost_airports = cluster(flight_graph, &airports);

    let elapsed_time = start.elapsed();

    println!("{:?}", topmost_airports);
    let _ = plot::plot_map(&airports);
    println!("Elapsed computing time: {:?}", elapsed_time);
}

fn find_bin_files(data_paths: Vec<String>) -> Vec<PathBuf> {
    data_paths.iter()
        .filter_map(|dir| PathBuf::from(dir).read_dir().ok())
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
                run_thread(nbr_airports, thread_bin_files, &finder_clone, &combiner_clone);
            });
        running_threads.push(running_thread);
    }

    for thread in running_threads {
        thread.join()
            .expect("Thread crashed unexpectedly");
    }

    let dissimilarity = combiner.lock()
        .unwrap()
        .to_float_vec();

    dissimilarity
}

fn cluster(flight_graph: Vec<f32>, airports: &Vec<Airport>) -> Vec<String>{
    clusterer::cluster(flight_graph, airports.len(), NBR_DEND_TOP)
        .into_iter()
        .map(|ind| format!("{}, {}", airports[ind].name, airports[ind].abr))
        .collect()

}

fn run_thread(nbr_airports: usize, bin_files: Vec<PathBuf>, airport_finder: &UsedAirportFinder,
              combiner: &Mutex<network::FlightCountNetwork>) {

    let mut file_graph = network::FlightCountNetwork::new(nbr_airports);
    for bin_path in bin_files {
        let flights = FlightsParser::parse(&bin_path);
        file_graph.add_flights(&flights, airport_finder);
    }

    // Add the total graph to the shared one
    combiner.lock()
        .unwrap()
        .add_network(file_graph);
}
