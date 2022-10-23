mod flights_parser;
mod airports;
mod network;
mod clusterer;
mod plot;

use std::{path::Path};

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

    let airports = airports::from_csv(&data_path.join("airports.csv"));
    let airport_finder = create_kd_tree(&airports);
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

fn process_flights(data_path: &Path, airport_finder: &KdTreeAirportFinder, airports: &Vec<Airport>) -> Vec<f32> {
    let start = Instant::now();
    println!("Parsing Networks...");
    // let graphs: Vec<network::FlightCountNetwork> = data_path.read_dir()
    //     .unwrap()
    //     .into_iter()
    //     .map(|entry| entry.unwrap().path())
    //     .filter(|path| {    // Filter to onli binary files TODO: Make better
    //         if let Some(ext) = path.extension() {
    //             ext == ".bin"
    //         } else { false }
    //     })
    //     .map(|path| dissimilarity_from_binary(path.as_path(), &airport_finder))
    //     .collect();

    let mut graphs = vec![];
    for entry in data_path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if let Some(ext) = path.extension() {
            if ext == "bin" {
                let file_graph = dissimilarity_from_binary(path.as_path(), &airport_finder, airports);
                graphs.push(file_graph);
            }
        }
    }
    println!("Parsing Networks... OK. Time: {:?}", start.elapsed());

    println!("Adding graphs...");
    let start = Instant::now();
    let mut graph = network::FlightCountNetwork::new(airports.len());
    for g in graphs {
        graph.add_network(g)
    }
    println!("Adding graphs... OK. Time: {:?}", start.elapsed());

    println!("Converting...");
    let start = Instant::now();
    let matrix = graph.connections()
        .iter()
        .map(|x| *x as f32)
        .collect();
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