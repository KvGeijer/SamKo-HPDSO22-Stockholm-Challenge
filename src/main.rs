mod flights_parser;
mod airports;
mod network;
mod clusterer;

use std::{io, path::Path};

use flights_parser::{FlightsParser};
use airports::{AirportFinder, KdTreeAirportFinder};

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

fn main(){

    let args = Args::parse();
    let data_path = Path::new(&args.path);

    let airport_finder = load_airports(&data_path);
    let flight_graph = process_flights(&data_path, &airport_finder);
    let topmost_airports = cluster(flight_graph, &airport_finder);

    println!("{:?}", topmost_airports);
}

fn load_airports(data_path: &Path) -> KdTreeAirportFinder {
    println!("loading airports..");
    let start = Instant::now();
    let airport_file = data_path.join("airports.csv");
    let airports = KdTreeAirportFinder::from_csv(airport_file.as_path());
    println!("Time: {:?}", start.elapsed());

    airports
}

fn process_flights(data_path: &Path, airport_finder: &KdTreeAirportFinder) -> Vec<f32> {
    let start = Instant::now();
    let mut graphs = vec![];
    println!("Parsing Networks...");
    for entry in data_path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        if let Some(ext) = path.extension() {
            if ext == "bin" {
                let flight_chunk = FlightsParser::parse(path.as_path());
                let mut graph_chunk = network::FlightCountNetwork::new(airport_finder.airport_count());
                graph_chunk.add_flights(&flight_chunk, airport_finder);
                graphs.push(graph_chunk);
            }
        }
    }
    println!("Time: {:?}", start.elapsed());

    println!("Adding results...");
    let start = Instant::now();
    let mut graph = network::FlightCountNetwork::new(airport_finder.airport_count());
    for g in graphs {
        graph.add_network(g)
    }
    println!("Converting...");
    let matrix = graph.connections().iter().map(|x| *x as f32).collect();
    println!("Time: {:?}", start.elapsed());

    matrix
}

fn cluster(flight_graph: Vec<f32>, airport_finder: &KdTreeAirportFinder) -> Vec<String>{
    println!("Clustering...");
    let start = Instant::now();
    let topmost = clusterer::cluster(flight_graph, airport_finder.airport_count(), 5);
    println!("Time: {:?}", start.elapsed());

    // TODO: Get real string names
    topmost.iter()
        .map(|ind| ind.to_string())
        .collect()

}