

use crate::airports::{AirportFinder};
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
        //println!("i: {}, j: {}, n: {}", i, j, self.n);
        let ind = (self.n - 1)*(i + 1) - (i*i - i)/2 - j - i;
        self.connections[ind] += 1;
    }

    pub fn add_flights(&mut self, flights: &[Flight], airports: &AirportFinder) {
        for flight in flights {
            let start = airports.closest(flight.from_lat, flight.from_long).id;
            let end = airports.closest(flight.to_lat, flight.to_long).id;
            self.add_flight(start - 1, end - 1);
        }
    }

    pub fn add_network(&mut self, other: FlightCountNetwork) {
        for (v1, v2) in self.connections.iter_mut().zip(other.connections) {
            *v1 += v2;
        }
    }

    pub fn connections(self) -> Vec<u32> {
        self.connections
    }
}

#[test]
fn flight_count_network_works() {
    let airports = vec![
        ("Stockholm".to_owned(), "ST".to_owned(), 59.3294, 18.0686, 1).into(),
        ("New York".to_owned(), "NY".to_owned(), 40.641766, -73.780968, 2).into(),
        ("Australia".to_owned(), "AU".to_owned(), -23.8067, 133.9017, 3).into(),
    ];
    let flights = &[
        //Gothenburg to New york
        Flight {from_lat: 57.6717, from_long: 11.9810, to_lat: 40.730610, to_long: -73.935242 },
        //Australia to Solna
        Flight {from_lat: -33.865143, from_long: 151.209900, to_lat: 59.36004, to_long: 18.00086}
    ];
    let mut network = FlightCountNetwork::new(airports.len());
    let finder = AirportFinder::new(airports);
    network.add_flights(flights, &finder);
    println!("{:?}", network.connections);
    assert_eq!(network.connections, vec![1, 1, 0])
}