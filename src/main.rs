mod flights_parser;
mod airports;
mod network;
mod clusterer;

use std::{io, path::Path};

use flights_parser::{FlightsParser};
use airports::{AirportFinder};

use clap::Parser;

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

fn main() -> io::Result<()> {

    let args = Args::parse();
    let data = Path::new(&args.path);

    let airport_file = data.join("airports.csv");
    let airports = AirportFinder::from_csv(airport_file.as_path());


    let mut graphs = vec![];

    for entry in data.read_dir()? {
        let path = entry?.path();
        if let Some(ext) = path.extension() {
            if ext == "bin" {
                let flight_chunk = FlightsParser::parse(path.as_path());
                let mut graph_chunk = network::FlightCountNetwork::new(airports.airport_count());
                graph_chunk.add_flights(&flight_chunk, &airports);
                graphs.push(graph_chunk);
            }
        }
    }

    let mut graph = network::FlightCountNetwork::new(airports.airport_count());
    for g in graphs {
        graph.add_network(g)
    }

    let matrix = graph.connections().iter().map(|x| *x as f32).collect();
    let res = clusterer::cluster(matrix, airports.airport_count(), 5);
    println!("{:?}", res);
    Ok(())
}

