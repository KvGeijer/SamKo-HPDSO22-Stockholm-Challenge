use std::path::Path;

use der_parser::ber::{parse_ber_octetstring, BerObject, BerObjectContent};

#[derive(Debug)]
pub struct Flight {
    pub from_lat: f32,
    pub from_long: f32,
    pub to_lat: f32,
    pub to_long: f32,
}

pub struct FlightsParser {}

impl FlightsParser {
    pub fn parse(file: &Path) -> Vec<Flight> {
        // NOTES: Each flight is encoded as an Universal constructed class containing an OctetString
        // with 20 octets. The first two octets are always (4, 8), as are the eleventh and twelfth.
        // Those probably denote the start of each pair in some way? Maybe the internal
        // representation they used for tuples.
        // The remaining 8 bytes of each pair represents two f32 values in sequence, encoded in
        // little endian.
        // We could just read the bytes directly to avoid th ecost of parsing.
        let mut flights: Vec<Flight> = vec![];

        let binary: Vec<u8> = std::fs::read(file).expect("Invalid binary file!");
        let mut stream = &binary[..];

        while let Ok((rest, ber)) = parse_ber_octetstring(stream) {
            stream = rest;
            flights.push(ber_to_flight(ber));
        }

        flights
    }
}

fn ber_to_flight(ber: BerObject) -> Flight {
    let bin_slice: &[u8] =
        match ber.content {
            BerObjectContent::OctetString(octet) => octet,
            other => panic!("ERROR, BER CONTENT HAS CHANGED: {:?}", other),
        };

    // To parse into f32 we have to copy the arrays which is not nice
    let from_lat = f32::from_le_bytes(bin_slice[2..6].try_into().unwrap());
    let from_long = f32::from_le_bytes(bin_slice[6..10].try_into().unwrap());
    let to_lat = f32::from_le_bytes(bin_slice[12..16].try_into().unwrap());
    let to_long = f32::from_le_bytes(bin_slice[16..20].try_into().unwrap());

    Flight {
        from_lat,
        from_long,
        to_lat,
        to_long,
    }
}
