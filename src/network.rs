
use crate::airports::{AirportFinder, Airport};
use crate::flights_parser::{Flight};

pub struct FlightCountNetwork {
    connections: Vec<u32>,
    n: usize,
}

impl FlightCountNetwork {
    pub fn new(n: usize) -> Self {
        return Self {
            n,
            connections: vec![0; n*(n - 1)/2]
        }
    }

    fn add_flight(&mut self, n1: usize, n2: usize) {
        let (i, j) = if n1 > n2 { (n2, n1) } else { (n1, n2) };
        let ind = self.n*(i + 1) - i*(i - 1)/2 - j - 1;
        self.connections[ind] += 1;
    }

    pub fn add_flights(&mut self, flights: &[Flight], airports: &AirportFinder) {
        for flight in flights {
            let start = airports.closest(flight.from_lat, flight.from_long).id;
            let end = airports.closest(flight.to_lat, flight.to_long).id;
            self.add_flight(start - 1, end - 1);
        }
    }
}
