//! Sort-of based on https://github.com/gunnarmorling/1brc/blob/main/src/main/java/dev/morling/onebrc/CalculateAverage.java
//! This is a really poor effort, designed for clarity and to highlight where performance will potentially suck.
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::collections::HashMap;
use std::time::Instant;

const FILE: &str = "/home/herbert/Rust/one_billion_rows/create_measurements/measurements.txt";

#[derive(Debug)]
struct Measurement {
    station: String,
    value: f32,
}

#[derive(Debug)]
struct Aggregator {
    min: f32,
    max: f32,
    sum: f64,
    count: u64,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self {
            min: f32::MAX,
            max: f32::MIN,
            sum: 0.0,
            count: 0,
        }
    }
}

#[derive(Debug)]
struct Collector {
    stations: HashMap<String, Aggregator>,
}

impl Collector {
    fn new() -> Self {
        Self {
            stations: HashMap::new(),
        }
    }

    fn collect(&mut self, measurement: &Measurement) {
        let entry = self.stations.entry(measurement.station.clone()).or_insert(Aggregator::default());
        entry.count += 1;
        entry.min = f32::min(entry.min, measurement.value);
        entry.max = f32::max(entry.max, measurement.value);
        entry.sum += measurement.value as f64;
    }
}

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let mut collector = Collector::new();
    let file = File::open(FILE)?;
    BufReader::new(file).lines().map(|l| {
        let l = l.unwrap();
        let mut split = l.split(';');
        Measurement {
            station: split.next().unwrap().trim().to_string(),
            value: split.next().unwrap().trim().parse().unwrap(),
        }
    }).for_each(|m| collector.collect(&m));

    let mut result: Vec<(&str, f32, f32, f32)> = collector.stations.iter().map(|(name, value)| {
        let mean = (value.sum / value.count as f64) as f32;
        (name.as_str(), value.min, value.max, mean)
    }).collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    print!("{{");
    result.iter().for_each(|(name, min, max, mean)| {
        print!("{name}={min:.1}/{max:.1}/{mean:.1}, ")
    });
    println!("}}");

    println!("Completed in {} seconds", start.elapsed().as_secs_f32());

    Ok(())
}
