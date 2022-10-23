use plotters::prelude::*;

use image::{imageops::FilterType, ImageFormat};

use std::fs::File;
use std::io::BufReader;

use crate::airports::Airport;

const OUT_FILE_NAME: &'static str = "result.png";

pub fn plot_map(airports: &Vec<Airport>) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Bitmap Example", ("sans-serif", 30))
        .margin(0)
        .set_label_area_size(LabelAreaPosition::Left, 0)
        .set_label_area_size(LabelAreaPosition::Bottom, 0)
        .build_cartesian_2d(0.0..1.0, 0.0..1.0)?;

    chart.configure_mesh().disable_mesh().draw()?;

    let (w, h) = chart.plotting_area().dim_in_pixel();
    let image = image::load(
        BufReader::new(
            File::open("data/earth.jpg").map_err(|e| {
                eprintln!("couldn't find image file earth.jpg");
                e
            })?),
        ImageFormat::Jpeg,
    )?
    .resize_exact(w - w / 10, h - h / 10, FilterType::Nearest);

    let elem: BitMapElement<_> = ((0.05_f64, 0.95_f64), image).into();

    chart.draw_series(std::iter::once(elem))?;
    chart.draw_series(airports.iter().map(|a| {
            let lat_0to1 = ((a.lat/90.0 + 1.0)/2.0) as f64;
            let long_0to1 = ((a.long/180.0 + 1.0)/2.0) as f64;
            Circle::new((long_0to1, lat_0to1), 5.0_f64, &RED)
    }))?;
    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
