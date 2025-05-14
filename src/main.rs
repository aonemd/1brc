use core::cmp::min;
use memmap2::Mmap;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
struct City {
    name: String,
    max: i32,
    min: i32,
    sum: i32,
    count: i32,
}

impl City {
    fn new(name: String, max: i32, min: i32) -> Self {
        Self {
            name,
            max,
            min,
            sum: max,
            count: 1,
        }
    }
}

fn fast_parse_float_to_int(data: &[u8]) -> i32 {
    let mut point = false;
    let mut negative = false;
    let mut result = 0;

    for &byte in data.iter() {
        if byte == b'-' {
            negative = true;
            continue;
        }

        if byte == b'.' {
            point = true;
            continue;
        }

        let digit = (byte - b'0') as i32;

        result = result * 10 + digit;
    }

    if negative {
        -result
    } else {
        result
    }
}

fn fast_parse_float(data: &[u8]) -> f32 {
    let mut result = 0.0;
    let mut point = false;
    let mut decimal = 2.0;
    let mut negative = false;
    let mut i = 0;
    if data[0] == b'-' {
        negative = true;
        i += 1;
    }
    while i < data.len() {
        let byte = data[i];
        if byte == b'.' {
            point = true;
            i += 1;
            continue;
        }
        let digit = (byte - b'0') as f32;
        if point {
            decimal *= 0.1;
            result += digit * decimal;
        } else {
            result = result * 10.0 + digit;
        }
        i += 1;
    }
    if negative {
        -result
    } else {
        result
    }
}

fn fast_float(input: &str) -> Result<f32, std::num::ParseFloatError> {
    let point = input.find('.').unwrap_or(input.len());
    let cutoff = min(point + 3, input.len());

    (&input[0..cutoff]).parse()
}

fn find_next_newline(data: &[u8]) -> Option<usize> {
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            return Some(i);
        }
    }
    None
}

fn main() {
    const NUMBER_OF_UNIQUE_STATIONS: usize = 10_000;

    let mut map: FxHashMap<String, City> = FxHashMap::default();

    let path = "./data/measurements.txt";
    // let path = "./data/measurements_100_000_000.txt";
    // let path = "./data/measurements_10.txt";

    let file = std::fs::File::open(path).expect("Failed to read file");
    let mmap = unsafe { Mmap::map(&file).expect("error mapping") };
    let mmap_len = mmap.len();
    let mmap = Arc::new(mmap);

    const _MAX_BLOCK_SIZE: usize = 16 * 1024 * 1024; // 2_097_152; //2M
    const THREAD_COUNT: usize = 8;
    let file_length = file.metadata().unwrap().len() as usize;
    let chunk_size: usize = file_length / THREAD_COUNT;

    let mut chunk_ranges = Vec::new();
    let mut start = 0;
    while start < mmap_len {
        let mut end = (start + chunk_size).min(mmap_len);

        if end < mmap_len {
            if let Some(newline_pos) = find_next_newline(&mmap[end..]) {
                end += newline_pos + 1; // include '\n'
            } else {
                end = mmap_len; // no newline until EOF
            }
        }

        chunk_ranges.push((start, end));
        start = end;
    }

    let (tx, rx) = channel();

    for i in 0..THREAD_COUNT {
        let tx = tx.clone();
        let mmap_chunk = Arc::clone(&mmap);

        let chunk_range = chunk_ranges[i];
        let start = chunk_range.0;
        let end = chunk_range.1;

        thread::spawn(move || {
            let mut local_map: FxHashMap<String, City> = FxHashMap::default();

            let mut last_newline = start;
            let mut last_split_at = 0;
            for i in start..end {
                let b = mmap_chunk[i];

                if b == b';' {
                    last_split_at = i;
                }

                if b == b'\n' {
                    let line_start = last_newline;
                    let line_end = i; // '\n' is excluded in all ranges since we're using [start..end]
                    last_newline = i + 1;

                    // split the line here
                    if last_split_at <= line_start {
                        continue;
                    }

                    let name = &mmap_chunk[line_start..last_split_at];
                    let temp = &mmap_chunk[(last_split_at + 1)..line_end];

                    let name = unsafe { std::str::from_utf8_unchecked(name) };
                    let temp = fast_parse_float_to_int(temp);

                    if local_map.contains_key(name) {
                        let city = local_map.get_mut(name).unwrap();
                        city.max = (city.max).max(temp);
                        city.min = (city.min).min(temp);
                        city.sum += temp;
                        city.count += 1;
                    } else {
                        local_map.insert(name.to_string(), City::new(name.to_string(), temp, temp));
                    }
                }
            }

            tx.send(local_map).expect("Failed to send data");
        });
    }

    drop(tx);

    for received_map in rx {
        for (name, city) in received_map {
            if map.contains_key(&name) {
                let global_city = map.get_mut(&name).unwrap();
                global_city.max = (global_city.max).max(city.max);
                global_city.min = (global_city.min).min(city.min);
                global_city.sum += city.sum;
                global_city.count += city.count;
            } else {
                map.insert(name, city);
            }
        }
    }

    let mut cities = map.into_values().collect::<Vec<City>>();
    cities.sort_by(|a, b| a.name.cmp(&b.name));

    let mut output = String::from("{");
    for (i, city) in cities.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        output.push_str(&format!(
            "{}={:.1}/{:.1}/{:.1}",
            city.name,
            (city.min as f32 / 10.0),
            ((city.sum / city.count) as f32 / 10.0).ceil(),
            (city.max as f32 / 10.0),
        ));
    }
    output.push('}');
    println!("{}", output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_integer() {
        let data = b"123";
        assert_eq!(fast_parse_float_to_int(data), 123);
    }

    #[test]
    fn test_negative_integer() {
        let data = b"-123";
        assert_eq!(fast_parse_float_to_int(data), -123);
    }

    #[test]
    fn test_positive_float() {
        let data = b"123.45";
        assert_eq!(fast_parse_float_to_int(data), 12345);
    }

    #[test]
    fn test_negative_float() {
        let data = b"-123.45";
        assert_eq!(fast_parse_float_to_int(data), -12345);
    }

    #[test]
    fn test_zero() {
        let data = b"0";
        assert_eq!(fast_parse_float_to_int(data), 0);
    }

    #[test]
    fn test_empty() {
        let data = b"";
        assert_eq!(fast_parse_float_to_int(data), 0);
    }
}
