/// Exercise 3 - Modules
/// a) Duplicate the exercise_2.rs to exercise_3.rs.
/// b) Move the filtering function to a new file called filter.rs.
/// c) Move the data structures and the behaviour of loading the json file to lib.rs.
use rustdemo::{load_cities, City};

pub fn largest_city(city_data: &Vec<City>, country_code: &str) {
    let mut largest_city: Option<(String, i64)> = None;
    for city in city_data {
        if city.fields.country_code == country_code {
            if let Some((_, largest_population)) = largest_city {
                if city.fields.population > largest_population {
                    largest_city = Some((city.fields.name.clone(), city.fields.population));
                }
            } else {
                largest_city = Some((city.fields.name.clone(), city.fields.population));
            }
        }
    }
    if let Some((name, population)) = largest_city {
        let population = population;
        println!("{}, pop: {}", name, population);
    } else {
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

        println!(
            "{}, {}, {}, {}",
            city.fields.name,
            city.fields.country_code,
            if let Some(admin1_code) = city.fields.admin1_code.clone() {
                admin1_code
            } else {
                "N/A".to_string()
            },
            city.fields.timezone
        );
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_cities()?;

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
