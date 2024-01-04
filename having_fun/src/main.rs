use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;
use memmap2::MmapOptions;

const FILE: &str = "/home/herbert/Rust/one_billion_rows/create_measurements/measurements.txt";
const NEWLINE: u8 = 10;
const SEMICOLON: u8 = 59;
const NUM_CPUS: usize = 32; // I only have 18!
const NUM_STATIONS: usize = 413;

#[derive(Debug)]
struct Aggregator {
    name: String,
    min: f32,
    max: f32,
    sum: f64,
    count: u64,
}

impl Default for Aggregator {
    fn default() -> Self {
        Self {
            name: String::new(),
            min: f32::MAX,
            max: f32::MIN,
            sum: 0.0,
            count: 0,
        }
    }
}

fn find_next_newline(start: usize, buffer: &[u8]) -> usize {
    let mut pos = start;
    while pos < buffer.len() {
        if buffer[pos] == NEWLINE {
            return pos+1;
        }
        pos += 1;
    }
    panic!("Oops - no line found, your algorithm is broken.")
}

fn scan_ascii_chunk(start: usize, end: usize, buffer: &[u8]) -> Vec<Aggregator> {
    let mut counter = HashMap::with_capacity(NUM_STATIONS);

    let mut pos = start;
    let mut line_start = start;
    let mut name_end = start;
    let mut val_start = start;
    while pos < end {
        match buffer[pos] {
            SEMICOLON => {
                // From line_start to here-1 is the name
                name_end = pos;
                val_start = pos + 2;
            }
            NEWLINE => {
                // This is the end of the line
                let station = &buffer[line_start..name_end];
                let value_ascii = &buffer[val_start..pos];
                let value_string = String::from_utf8_lossy(value_ascii);
                let value: f32 = value_string.parse().unwrap();
                let entry = counter.entry(station).or_insert(Aggregator::default());
                if entry.name.is_empty() {
                    entry.name = String::from_utf8_lossy(station).to_string();
                }
                entry.max = f32::max(value, entry.max);
                entry.min = f32::min(value, entry.min);
                entry.sum += value as f64;
                entry.count += 1;

                // Therefore the next line starts at the next character
                line_start = pos + 1;
            }
            _ => {}
        }

        pos += 1;
    }
    counter.into_iter().map(|(_k, v)| v).collect()
}

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let file = File::open(FILE)?;
    let mapped_file = unsafe { MmapOptions::new().map(&file)? };
    let size = mapped_file.len();

    // Divide the mapped memory into roughly equal chunks. We'll store
    // a starting point and ending point for each chunk. Starting
    // points are adjusted to seek forward to the next newline.
    let chunk_length = size / NUM_CPUS;
    let mut starting_points: Vec<usize> = (0 .. NUM_CPUS)
        .map(|n| n * chunk_length)
        .collect();
    for i in 1..NUM_CPUS {
        starting_points[i] = find_next_newline(starting_points[i], &mapped_file);
    }

    let mut ending_points = vec![0usize; NUM_CPUS];
    for i in 0..NUM_CPUS-1 {
        ending_points[i] = starting_points[i+1];
    }
    ending_points[NUM_CPUS-1] = size;

    // Using a scoped pool to make it easy to share the immutable data from above.
    // Scan each segment to find station names and values.
    let mut result = Vec::with_capacity(NUM_STATIONS);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(NUM_CPUS);
        for thread in 0..NUM_CPUS {
            let start = starting_points[thread];
            let end = ending_points[thread];
            let buffer = &mapped_file;
            let handle = scope.spawn(move || {
                scan_ascii_chunk(start, end, &buffer)
            });
            handles.push(handle);
        }

        // Aggregate the results
        for handle in handles {
            let chunk_result = handle.join().unwrap();
            if result.is_empty() {
                result.extend(chunk_result);
            } else {
                chunk_result.into_iter().for_each(|v| {
                    if let Some(agg) = result.iter_mut().find(|a| a.name == v.name) {
                        agg.sum += v.sum;
                        agg.count += v.count;
                        agg.max = f32::max(agg.max, v.max);
                        agg.min = f32::min(agg.min, v.min);
                    } else {
                        result.push(v);
                    }
                });
            }
        }
    });

    result.sort_unstable_by(|a,b| a.name.cmp(&b.name));
    //assert_eq!(result.len(), NUM_STATIONS);

    print!("{{");
    result.iter().for_each(|v| {
        let mean = v.sum / v.count as f64;
        print!("{}={:.1}/{:.1}/{mean:.1}, ", v.name, v.min, v.max);
    });
    println!("}}");

    println!("Completed in {} seconds", start.elapsed().as_secs_f32());
    Ok(())
}
