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
    println!("Loading airports..");
    let start = Instant::now();
    let airport_file = data_path.join("airports.csv");
    let airports = KdTreeAirportFinder::from_csv(airport_file.as_path());
    println!("Loading airport... OK. Time: {:?}", start.elapsed());

    airports
}

fn process_flights(data_path: &Path, airport_finder: &KdTreeAirportFinder) -> Vec<f32> {
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
                let file_graph = dissimilarity_from_binary(path.as_path(), &airport_finder);
                graphs.push(file_graph);
            }
        }
    }
    println!("Parsing Networks... OK. Time: {:?}", start.elapsed());

    println!("Adding graphs...");
    let start = Instant::now();
    let mut graph = network::FlightCountNetwork::new(airport_finder.airport_count());
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

fn cluster(flight_graph: Vec<f32>, airport_finder: &KdTreeAirportFinder) -> Vec<String>{
    println!("Clustering...");
    let start = Instant::now();
    let topmost = clusterer::cluster(flight_graph, airport_finder.airport_count(), 5);
    println!("Clustering... OK. Time: {:?}", start.elapsed());

    // TODO: Get real string names
    topmost.iter()
        .map(|ind| ind.to_string())
        .collect()

}

fn dissimilarity_from_binary(bin_path: &Path, airport_finder: &KdTreeAirportFinder)
                                        -> network::FlightCountNetwork {

    println!("Parsing binary file...");
    let start = Instant::now();
    let flight_chunk = FlightsParser::parse(bin_path);
    println!("Parsing binary file... OK. Time: {:?}", start.elapsed());

    println!("Locating airports...");
    let start = Instant::now();
    let mut dissimilarity_graph = network::FlightCountNetwork::new(airport_finder.airport_count());
    dissimilarity_graph.add_flights(&flight_chunk, airport_finder);
    println!("Locating airports... OK. Time: {:?}", start.elapsed());




    dissimilarity_graph
}