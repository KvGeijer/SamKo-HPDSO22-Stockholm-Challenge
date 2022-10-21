mod flights_parser;
mod airports;
mod network;

use flights_parser::{FlightsParser, Flight};
use airports::{AirportFinder};

fn main() {
    let _flights: Vec<Flight> = FlightsParser::parse("./data/dat.bin");
    let finder = AirportFinder::from_csv("data/airports.csv");
    let airport = finder.closest(59.3518061, 18.1344461); //(59.3294, 18.0686);//
    println!("{:?}", airport)
}
