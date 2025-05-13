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

fn main() {
    const NUMBER_OF_UNIQUE_STATIONS: usize = 10_000;

    let mut map: FxHashMap<String, City> = FxHashMap::default();

    let path = "./data/measurements.txt";
    // let path = "./data/measurements_100_000_000.txt";
    // let path = "./data/measurements_10.txt";

    let file = std::fs::File::open(path).expect("Failed to read file");
    let mmap = unsafe { Mmap::map(&file).expect("error mapping") };
    let mmap = Arc::new(mmap);

    let file_length = file.metadata().unwrap().len() as usize;
    const _MAX_BLOCK_SIZE: usize = 16 * 1024 * 1024; // 2_097_152; //2M
    const THREAD_COUNT: usize = 8;
    let chunk_size: usize = file_length / THREAD_COUNT;

    let (tx, rx) = channel();

    for i in 0..THREAD_COUNT {
        let tx = tx.clone();
        let mmap_chunk = Arc::clone(&mmap);

        thread::spawn(move || {
            let mut local_map: FxHashMap<String, City> = FxHashMap::default();

            let start = i * chunk_size;
            let end = (((i + 1) * chunk_size) + 100).min(file_length);
            for line in mmap_chunk[start..end].split(|b| *b == b'\n') {
                if line.is_empty() {
                    continue;
                }

                let mut line_parts = line.split(|bb| *bb == b';');
                let name = unsafe { std::str::from_utf8_unchecked(line_parts.next().unwrap()) };
                let temp = fast_parse_float_to_int(line_parts.next().unwrap());

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
