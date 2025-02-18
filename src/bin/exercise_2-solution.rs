/// 1. Move the read_to_string and serde_json::from_str calls into a separate
///    function.
/// 2. Write a function which prints the largest city for a given a the
///    list of cities and a given country.
/// 3. Write a function that takes a "filter" parameter, which is an enum
///    of multiple variants: CountryCode, Admin1Code or TimeZone. Print
///    the name of all cities that match that filter.

use std::fs;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct City {
    pub datasetid: String,
    pub recordid: String,
    pub fields: CityData,
    pub record_timestamp: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CityData {
    pub coordinates: [f64; 2],
    pub cou_name_en: Option<String>,
    pub label_en: Option<String>,
    pub feature_code: String,
    pub population: i64,
    pub dem: i64,
    pub geoname_id: String,
    pub name: String,
    pub admin1_code: Option<String>,
    pub admin2_code: Option<String>,
    pub admin3_code: Option<String>,
    pub admin4_code: Option<String>,
    pub feature_class: String,
    pub country_code: String,
    pub timezone: String,
    pub modification_date: String,
}

pub fn load_city_data() -> Result<Vec<City>, Box<dyn std::error::Error>> {
    let file_name = "cities100k.json";
    let json_data = fs::read_to_string(file_name)?;
    let city_data = serde_json::from_str::<Vec<City>>(&json_data)?;

    Ok(city_data)
}

pub fn largest_city(city_data : &Vec<City>, country_code : &str) {
    let mut largest_city: Option<(String, i64)> = None;
    for city in city_data {
        if city.fields.country_code == country_code {
            if let Some((_, largest_population)) = largest_city {
                if city.fields.population > largest_population {
                    largest_city = Some((city.fields.name.clone(), city.fields.population));
                }
            }
            else {
                largest_city = Some((city.fields.name.clone(), city.fields.population));
            }
        }
    };
    if let Some((name, population)) = largest_city {
        let population = population;
        println!("{}, pop: {}", name, population);
    }
    else {
        println!("Largest city not found");
    }
}


pub enum Filter {
    CountryCode(String),
    Admin1Code(String),
    TimeZone(String),
}

fn filter_cities(city_data: &Vec<City>, filter: Filter) {
    for city in city_data {
        match &filter {
            Filter::CountryCode(v) => {
                if &city.fields.country_code != v {
                    continue;
                }
            }
            Filter::Admin1Code(v) => {
                let code = match &city.fields.admin1_code {
                    Some(x) => x,
                    None => continue,
                };

                if code != v {
                    continue;
                }
            }
            Filter::TimeZone(v) => {
                if &city.fields.timezone != v {
                    continue;
                }
            }
        }

        println!("{}, {}, {}, {}",
            city.fields.name,
            city.fields.country_code,
            if let Some(admin1_code) = city.fields.admin1_code.clone() { admin1_code } else { "N/A".to_string() },
        city.fields.timezone);
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_city_data()?;

    let sweden: String = "SE".to_string();

    print!("Largest city in Sweden: ");
    largest_city(&cities, &sweden);

    print!("Largest city in Tonga: ");
    largest_city(&cities, "to");

    println!();
    println!("Cities in CET:");
    println!("==================");
    let filter = Filter::TimeZone("Europe/Stockholm".to_string());
    filter_cities(&cities, filter);
    println!();
    println!();
    println!("Cities in Arizona:");
    println!("======================");
    let filter = Filter::Admin1Code("AZ".to_string());
    filter_cities(&cities, filter);
    println!();
    println!();
    println!("Cities in Taiwan:");
    println!("=====================");
    let filter = Filter::CountryCode("TW".to_string());
    filter_cities(&cities, filter);

    Ok(())
}
