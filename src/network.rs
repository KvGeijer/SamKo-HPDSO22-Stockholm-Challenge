use crate::airports::AirportFinder;
use crate::flights_parser::Flight;


pub struct FlightCountNetwork {
    connections: Vec<u32>,
    n: usize,
}


impl FlightCountNetwork {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            connections: vec![0; n*(n - 1)/2]
        }
    }

    fn add_flight(&mut self, from: usize, to: usize) {
        if from != to {
            let (row, col) = if from > to { (to, from) } else { (from, to) };
            let ind = row*(2*self.n - row - 1)/2 + col - row - 1;
            self.connections[ind] += 1;
        }
    }

    pub fn add_flights<T: AirportFinder>(&mut self, flights: &[Flight], airports: &T) {
        for flight in flights {
            let start = airports.closest_ind(flight.from_lat, flight.from_long);
            let end = airports.closest_ind(flight.to_lat, flight.to_long);
            self.add_flight(start, end);
        }
    }

    pub fn add_network(&mut self, other: FlightCountNetwork) {
        for (v1, v2) in self.connections.iter_mut().zip(other.connections) {
            *v1 += v2;
        }
    }

    pub fn to_dissimilarity_vec(&self) -> Vec<f32> {
        // Convert the network to a dissimilarity vector represneting a upper triangular matrix
        //
        // The similarity metric is defined as the number of connecting flights between two clusters
        // which means the dissimilarity shuld be inversely proportional to it. We also want to
        // avoid negative distances, so we negate the values and shift up all of them to be positive

        let shift = *self.connections
            .iter()
            .max()
            .unwrap();

        self.connections
            .iter()
            .map(|unsigned| (shift  - *unsigned) as f32)
            .collect()
    }
}


#[cfg(test)]
mod test {
    use crate::airports::{Airport, KdTreeAirportFinder};
    use super::*;

    #[test]
    fn flight_count_network_works() {
        let airports = vec![
            Airport {name: "Stockholm".to_owned(), abr: "ST".to_owned(), lat: 59.3294, long: 18.0686, id: 1},
            Airport {name: "New York".to_owned(), abr: "NY".to_owned(), lat: 40.641766, long: -73.780968, id: 2},
            Airport {name: "Australia".to_owned(), abr: "AU".to_owned(), lat: -23.8067, long: 133.9017, id: 3},
        ];
        let flights = &[
            //Gothenburg to New york
            Flight {from_lat: 57.6717, from_long: 11.9810, to_lat: 40.730610, to_long: -73.935242 },
            //Australia to Solna
            Flight {from_lat: -33.865143, from_long: 151.209900, to_lat: 59.36004, to_long: 18.00086}
        ];
        let mut network = FlightCountNetwork::new(airports.len());
        let finder = KdTreeAirportFinder::new(&airports);
        network.add_flights(flights, &finder);
        println!("{:?}", network.connections);
        assert_eq!(network.connections, vec![1, 1, 0])
    }

    #[test]
    fn index_map_test() {
        // Make a 5x5 matrix and insert things at a few points, then make sure it is correct
        // - 1 2 0 3 0
        // - - 4 0 0 5
        // - - - 6 0 7
        // - - - - 8 9
        // - - - - - 10
        // - - - - - -

        let mut network = FlightCountNetwork::new(6);
        for _ in 0..1 { network.add_flight(0, 1); }
        for _ in 0..2 { network.add_flight(0, 2); }
        for _ in 0..3 { network.add_flight(0, 4); }
        for _ in 0..4 { network.add_flight(1, 2); }
        for _ in 0..5 { network.add_flight(1, 5); }
        for _ in 0..6 { network.add_flight(2, 3); }
        for _ in 0..7 { network.add_flight(2, 5); }
        for _ in 0..8 { network.add_flight(3, 4); }
        for _ in 0..9 { network.add_flight(3, 5); }
        for _ in 0..10 { network.add_flight(4, 5); }

        let theoretical = vec![1,2,0,3,0,4,0,0,5,6,0,7,8,9,10];
        assert_eq!(theoretical, network.connections);
    }
}