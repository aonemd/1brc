use core::cmp::min;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
struct City {
    name: String,
    max: f32,
    min: f32,
    sum: f32,
    count: i32,
}

impl City {
    fn new(name: String, max: f32, min: f32) -> Self {
        Self {
            name,
            max,
            min,
            sum: 0f32,
            count: 1,
        }
    }
}

fn fast_parse_float(data: &[u8]) -> f32 {
    let mut result = 0.0;
    let mut point = false;
    let mut decimal = 1.0;
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
    let mut map: HashMap<String, City> = HashMap::new();

    let path = "./data/measurements.txt";
    let path = "./data/measurements_100_000_000.txt";
    // let path = "./data/measurements_10.txt";

    let file = std::fs::File::open(path).expect("Failed to read file");
    let mut reader = BufReader::new(file);
    const BLOCK_SIZE: usize = 16 * 1024 * 1024; // 2_097_152; //2M
    let mut buffer = vec![0_u8; BLOCK_SIZE];
    let mut left_over_bytes: Vec<u8> = vec![];

    let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

    std::thread::spawn(move || loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }

        left_over_bytes.extend_from_slice(&buffer);

        sender.send(left_over_bytes.clone()).unwrap();

        if let Some(last_newline_index) = buffer.iter().rposition(|&b| b == b'\n') {
            left_over_bytes = buffer[last_newline_index + 1..].to_vec();
        }
    });

    for line_data in receiver {
        let mut start = 0;
        let mut name_end = 0;
        for (i, &byte) in line_data.iter().enumerate() {
            if byte == b';' {
                name_end = i;
            }

            if byte == b'\n' && start < name_end {
                // includes the newline character but the slicing does not since it's
                // non-inclusive
                let end = i;

                let name = unsafe { std::str::from_utf8_unchecked(&line_data[start..name_end]) };
                let temp = fast_parse_float(&line_data[name_end + 1..end]);

                if map.contains_key(name) {
                    let city = map.get_mut(name).unwrap();
                    city.max = (city.max).max(temp);
                    city.min = (city.max).min(temp);
                    city.sum += temp;
                    city.count += 1;
                } else {
                    map.insert(name.to_string(), City::new(name.to_string(), temp, temp));
                }

                start = end + 1;
                name_end = 0;
            }
        }
    }

    let mut cities = map.into_values().collect::<Vec<City>>();
    cities.sort_by(|a, b| a.name.cmp(&b.name));

    let mut city_strings = Vec::new();
    for city in &cities {
        let city_string = format!(
            "{}={}/{}/{}",
            city.name,
            city.min,
            city.sum / city.count as f32,
            city.max,
        );
        city_strings.push(city_string);
    }

    let output = format!("{{{}}}", city_strings.join(", "));
    println!("{}", output);
}
