mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{io, thread, path::Path};
use std::sync::{Mutex, Arc};

use flights_parser::{FlightsParser};
use airports::{KdTreeAirportFinder, Airport};

use clap::Parser;
use std::time::Instant;

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

    let airports = Arc::new(airports::from_csv(&data_path.join("airports.csv")));
    let airport_finder = Arc::new(create_kd_tree(&airports));
    let flight_graph = process_flights(&data_path, &airport_finder, &airports);
    let topmost_airports = cluster(flight_graph, airports.len());

    println!("{:?}", topmost_airports);
    let _ = plot::plot_map(&airports);
}

fn create_kd_tree(airports: &Vec<Airport>) -> KdTreeAirportFinder {
    println!("Loading airports..");
    let start = Instant::now();
    let kdtree = KdTreeAirportFinder::new(airports);
    println!("Loading airport... OK. Time: {:?}", start.elapsed());
    kdtree
}

fn process_flights(data_path: &Path, airport_finder: &Arc<KdTreeAirportFinder>, airports: &Arc<Vec<Airport>>) -> Vec<f32> {
    let start = Instant::now();
    println!("Parsing and adding Networks...");

    let combiner = Arc::new(Mutex::new(network::FlightCountNetwork::new(airports.len())));
    let mut running_threads = vec![];

    // Iterate over binary files and convert them into matrices
    // Multithreaded: Each threads adds into the combiner network with Mutex
    for entry in data_path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if let Some(ext) = path.extension() {
            if ext == "bin" {
                let combiner_clone = combiner.clone();
                let finder_clone = airport_finder.clone();
                let airports_clone = airports.clone();
                let running_thread =
                    thread::spawn(move || {
                        let file_graph = dissimilarity_from_binary(path.as_path(),
                                                                   &finder_clone,
                                                                   &airports_clone);
                        combiner_clone.lock()
                            .unwrap()
                            .add_network(file_graph);
                    });
                running_threads.push(running_thread);
            }
        }
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

fn dissimilarity_from_binary(bin_path: &Path, airport_finder: &KdTreeAirportFinder, airports: &Vec<Airport>)
                                        -> network::FlightCountNetwork {

    println!("Parsing binary file...");
    let start = Instant::now();
    let flight_chunk = FlightsParser::parse(bin_path);
    println!("Parsing binary file... OK. Time: {:?}", start.elapsed());

    println!("Locating airports...");
    let start = Instant::now();
    let mut dissimilarity_graph = network::FlightCountNetwork::new(airports.len());
    dissimilarity_graph.add_flights(&flight_chunk, airport_finder);
    println!("Locating airports... OK. Time: {:?}", start.elapsed());
    dissimilarity_graph
}