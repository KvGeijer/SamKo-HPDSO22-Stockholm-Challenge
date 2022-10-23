use std::path::Path;

use quick_csv;
use kd_tree::{KdPoint, KdTree};
use typenum;

#[derive(Debug)]
pub struct Airport {
    pub name: String,
    pub abr: String,
    pub pos: [f32; 3],
    pub id: usize,
}

impl From<(String, String, f32, f32, usize)> for Airport {
    fn from(v: (String, String, f32, f32, usize)) -> Self {
        let pos = lat_long_to_point(v.2, v.3);
        Self {
            name: v.0,
            abr: v.1,
            id: v.4,
            pos,
        }
    }
}

impl KdPoint for Airport {
    type Scalar = f32;
    type Dim = typenum::U3;
    fn at(&self, k: usize) -> f32 { self.pos[k] }
}

pub struct AirportFinder {
    tree: KdTree<Airport>,
    len: usize,
}

impl AirportFinder {
    pub fn new(airports: Vec<Airport>) -> Self {
        Self { len: airports.len(), tree: KdTree::build_by_ordered_float(airports) }
    }

    pub fn airport_count(&self) -> usize {
        self.len
    }

    pub fn from_csv(path: &Path) -> Self {
        let csv = quick_csv::Csv::from_file(path).expect("Could not find airport location file.");
    let airports = csv.into_iter()
        .skip(1)
        .map(|r| {
            r.expect("Could not parse line in airport location file.")
                .decode::<(String, String, f32, f32, usize)>()
                .expect("Could not decode line into expected format in airport location file.")
                .into()
        })
        .collect();
        Self::new(airports)
    }

    pub fn closest(&self, lat: f32, long: f32) -> &Airport {
        let point = lat_long_to_point(lat, long);
        self.tree.nearest(&point).expect("embty").item
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

