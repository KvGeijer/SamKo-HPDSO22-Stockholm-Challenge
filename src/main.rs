mod flights_parser;

use flights_parser::{FlightsParser, Flight};

fn main() {
    let _flights: Vec<Flight> = FlightsParser::parse("./data/dat.bin");
}
