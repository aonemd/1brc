use core::cmp::min;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};

#[derive(Debug)]
struct City {
    name: String,
    max: f64,
    min: f64,
    sum: f64,
    count: i32,
}

impl City {
    fn new(name: String, max: f64, min: f64) -> Self {
        Self {
            name,
            max,
            min,
            sum: 0f64,
            count: 1,
        }
    }
}

fn fast_float(input: &str) -> Result<f64, std::num::ParseFloatError> {
    let point = input.find('.').unwrap_or(input.len());
    let cutoff = min(point + 3, input.len());

    (&input[0..cutoff]).parse()
}

fn main() {
    let mut map: HashMap<String, City> = HashMap::new();

    let path = "./data/measurements.txt";
    // let path = "./data/measurements_1000_000.txt";

    let file = std::fs::File::open(path).expect("Failed to read file");
    let mut reader = BufReader::new(file);
    const BLOCK_SIZE: usize = 16 * 1024 * 1024; // 2_097_152; //2M
    let mut buffer = vec![0_u8; BLOCK_SIZE];

    let mut left_over_bytes: Vec<u8> = vec![];

    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }

        left_over_bytes.extend_from_slice(&buffer);
        left_over_bytes.split(|b| b == &0xA).for_each(|line| {
            if let Ok(str_line) = std::str::from_utf8(line) {
                if let Some((name, temp)) = str_line.split_once(';') {
                    if let Ok(temp) = fast_float(temp) {
                        if map.contains_key(name) {
                            let city = map.get_mut(name).unwrap();
                            city.max = (city.max).max(temp);
                            city.min = (city.max).min(temp);
                            city.sum += temp;
                            city.count += 1;
                        } else {
                            map.insert(name.to_string(), City::new(name.to_string(), temp, temp));
                        }
                    };
                }
            }
        });

        if let Some(last_newline_index) = buffer.iter().rposition(|&b| b == b'\n') {
            left_over_bytes = buffer[last_newline_index..].to_vec();
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
            city.sum / city.count as f64,
            city.max,
        );
        city_strings.push(city_string);
    }

    let output = format!("{{{}}}", city_strings.join(", "));
    println!("{}", output);
}
