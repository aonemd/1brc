use core::cmp::min;
use memmap2::Mmap;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::usize;

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
            sum: 0,
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

    // let path = "./data/measurements.txt";
    // let path = "./data/measurements_100_000_000.txt";
    let path = "./data/measurements_10.txt";

    let file = std::fs::File::open(path).expect("Failed to read file");
    let mmap = unsafe { Mmap::map(&file).expect("error mapping") };
    let file_length = file.metadata().unwrap().len() as usize;

    const BLOCK_SIZE: usize = 16 * 1024 * 1024; // 2_097_152; //2M
    const THREAD_COUNT: usize = 8;
    let chunk_size: usize = file_length / THREAD_COUNT;
    //
    // let mut pool = vec![];
    let mut start_offsets: Vec<(usize, usize)> = vec![];
    let mut start = 0;
    for i in 0..THREAD_COUNT {
        if start > file_length {
            break;
        }
        // let mut start = file_length * i / THREAD_COUNT;
        // while start < file_length && mmap[start] != b'\n' {
        //     start += 1;
        // }

        let mut end = min(start + chunk_size, file_length);
        // let mut end = if i == THREAD_COUNT - 1 {
        //     file_length - 1
        // } else {
        //     file_length * (i + 1) / THREAD_COUNT
        // };

        while end < file_length - 1 && mmap[end] != b'\n' {
            end += 1;
        }

        start_offsets.push((start, end));

        start = end + 1
    }

    for (start, end) in start_offsets {
        // for i in 0..THREAD_COUNT {
        // let start = start_offsets[i].0;
        // let end = start_offsets[i].1;

        println!(
            "{} -> {} = {:?}",
            start,
            end,
            String::from_utf8_lossy(&mmap[start..end])
        );

        // ab;12\nc;13
        // 10
        let mut line_start = start;
        let mut name_end = 0;
        for (index, &byte) in mmap[start..end].iter().enumerate() {
            // while byte == b'\n' {
            //     continue;
            // }

            println!("{}", char::from(byte));

            if byte == b';' {
                name_end = index;
                continue;
            }

            if (byte == b'\n' || index == end - 1) && line_start < name_end {
                let line_end = index + 1;

                // start .. name_end
                let name = unsafe { std::str::from_utf8_unchecked(&mmap[line_start..name_end]) };
                // name_end+1 .. end
                // let temp = fast_parse_float_to_int(&mmap[name_end + 1..end]);
                let temp = unsafe { std::str::from_utf8_unchecked(&mmap[name_end + 1..line_end]) };

                println!("{name} => {temp}");

                line_start = line_end;
                name_end = 0;
            }
        }

        // let thr = std::thread::spawn(move || {});
        // pool.push(thr);
    }

    // for thr in pool {
    //     thr.join().unwrap();
    // }

    // for line in mmap.split(|b| *b == b'\n') {
    //     if line.is_empty() {
    //         continue;
    //     }
    //
    //     let line_parts: Vec<&[u8]> = line.split(|bb| *bb == b';').collect();
    //     let name = unsafe { std::str::from_utf8_unchecked(line_parts[0]) };
    //     let temp = fast_parse_float_to_int(line_parts[1]);
    //
    //     if map.contains_key(name) {
    //         let city = map.get_mut(name).unwrap();
    //         city.max = (city.max).max(temp);
    //         city.min = (city.max).min(temp);
    //         city.sum += temp;
    //         city.count += 1;
    //     } else {
    //         map.insert(name.to_string(), City::new(name.to_string(), temp, temp));
    //     }
    // }
    //
    // let mut cities = map.into_values().collect::<Vec<City>>();
    // cities.sort_by(|a, b| a.name.cmp(&b.name));
    //
    // let mut city_strings = Vec::new();
    // for city in &cities {
    //     let city_string = format!(
    //         "{}={:.1}/{:.1}/{:.1}",
    //         city.name,
    //         (city.min as f32 / 10.0),
    //         ((city.sum / city.count) as f32 / 10.0).ceil(),
    //         (city.max as f32 / 10.0),
    //     );
    //     city_strings.push(city_string);
    // }
    //
    // let output = format!("{{{}}}", city_strings.join(", "));
    // println!("{}", output);
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
