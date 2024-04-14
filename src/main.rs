use std::collections::HashMap;
use std::io::{prelude::*, BufReader};

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

fn main() {
    let mut map: HashMap<String, City> = HashMap::new();

    // let input = "./data/measurements.txt";
    let input = "./data/measurements_1000_000.txt";

    let input = std::fs::File::open(input).expect("Failed to read file");
    let reader = BufReader::new(input);
    for line in reader.lines() {
        if let Some((name, temp)) = line.unwrap().split_once(';') {
            let temp = temp.parse::<f32>().unwrap();

            if map.contains_key(name) {
                let city = map.get_mut(name).unwrap();
                city.max = (city.max).max(temp);
                city.min = (city.max).min(temp);
                city.sum += temp;
                city.count += 1;
            } else {
                map.insert(name.to_string(), City::new(name.to_string(), temp, temp));
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
