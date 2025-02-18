/// Write a program that prints the largest city for each country
///
/// Useful snippets:
///   use std::collections::HashMap;
///   let mut countries = HashMap::new();
///   countries.entry("Countryname").and_modify(|stored_item| *stored_item = item).or_insert(item);

use std::collections::HashMap;
use rustdemo::helpers::exercise_4::city_parser::*;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_city_data()?;

    let cities = largest_city_for_each_country(cities);
    let mut cities: Vec<CityData> = cities.iter().map(|(_, city)| city.clone()).collect();

    cities.sort_by(|a,b| {
        let country_name_a = a.country_name_eng();
        let country_name_b = b.country_name_eng();
        if country_name_a != country_name_b {
            country_name_a.cmp(&country_name_b)
        }
        else {
            a.name.cmp(&b.name)
        }
    });

    for city in cities {
        println!("{:<40}: {:<25}", city.country_name_eng(), city.name);
    }

    Ok(())
}

fn largest_city_for_each_country(city_data: Vec<CityData>) -> HashMap<String, CityData> {
    let mut largest_cities = HashMap::<String, CityData>::new();
    for city in city_data {
        let country_name = city.country_name_eng();
        largest_cities
            .entry(country_name.to_string())
            .and_modify(|current_city| {
                if city.population > current_city.population {
                    *current_city = city.clone();
                }
            })
            .or_insert(city.clone());
    };
    largest_cities
}