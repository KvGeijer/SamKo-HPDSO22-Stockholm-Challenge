use std::collections::HashMap;

use quick_csv;
use kd_tree::KdMap;

#[derive(Debug)]
pub struct Airport {
    pub name: String,
    pub abr: String,
    pub lat: f32,
    pub long: f32,
    pub id: usize,
}

pub fn from_csv(path: &str) -> Vec<Airport> {
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
    #[allow(dead_code)]
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
        self.tree.nearest(&point)
            .expect("Kd tree is empty")
            .item
            .1
    }
}


pub struct HashAirportFinder {
    map: HashMap<[u8; 8], usize>
}

impl HashAirportFinder {
    #[allow(dead_code)]
    pub fn new(airports: &Vec<Airport>) -> Self {
        let mut map = HashMap::new();
        for (i, airport) in airports.iter().enumerate() {

            let key = Self::bucket_coord(airport.lat, airport.long);
            let res = map.insert(key, i);
            if let Some(_) = res {
                panic!("Two different airports had same bucket hash! Internal error")
            }
        }
        Self { map }
    }

    fn bucket_coord(lat: f32, long: f32) -> [u8; 8] {
        unsafe {std::mem::transmute((lat, long))}
    }
}

impl AirportFinder for HashAirportFinder {
    fn closest_ind(&self, lat: f32, long: f32) -> usize {
        let bucket = Self::bucket_coord(lat, long);
        match self.map.get(&bucket) {
            Some(&v) => v,
            None => panic!("Cound not find bucket, change code to KdTreeFinder")
        }
    }
}


fn lat_long_to_point(lat: f32, long: f32) -> [f32; 3] {
    let lo = long.to_radians();
    let la = lat.to_radians();
    let x = lo.cos()*la.sin();
    let y = lo.sin()*la.sin();
    let z = la.cos();
    [x, y, z]
}

