use std::path::Path;

use quick_csv;
use kd_tree::{KdMap};

#[derive(Debug)]
pub struct Airport {
    pub name: String,
    pub abr: String,
    pub lat: f32,
    pub long: f32,
    pub id: usize,
}

pub fn from_csv(path: &Path) -> Vec<Airport> {
    let csv = quick_csv::Csv::from_file(path).expect("Could not find airport location file.");
    csv.into_iter()
        .skip(1)
        .map(|r| {
            let v = r.expect("Could not parse line in airport location file.")
                .decode::<(String, String, f32, f32, usize)>()
                .expect("Could not decode line into expected format in airport location file.");
            Airport {
                name: v.0,
                abr: v.1,
                lat: v.2,
                long: v.3,
                id: v.4,
            }
        })
        .collect()
}

pub trait AirportFinder {
    fn closest_ind(&self, lat: f32, long: f32) -> usize;
}

pub struct KdTreeAirportFinder {
    tree: KdMap<[f32; 3], usize>,
}

impl KdTreeAirportFinder {
    pub fn new(airports: &Vec<Airport>) -> Self {
        let spatial_ind = airports.iter().enumerate().map(|(i, a)|
            (lat_long_to_point(a.lat, a.long), i)
        ).collect();
        Self {
            tree: KdMap::build_by_ordered_float(spatial_ind),
        }
    }
}

impl AirportFinder for KdTreeAirportFinder {
    fn closest_ind(&self, lat: f32, long: f32) -> usize {
        let point = lat_long_to_point(lat, long);
        self.tree.nearest(&point).expect("embty").item.1
    }
}

fn lat_long_to_point(lat: f32, long: f32) -> [f32; 3] {
    //TODO if lat/long are very close to read data, then we could just do rounding + hashmap
    let lo = long.to_radians();
    let la = lat.to_radians();
    let x = lo.cos()*la.sin();
    let y = lo.sin()*la.sin();
    let z = la.cos();
    [x, y, z]
}

