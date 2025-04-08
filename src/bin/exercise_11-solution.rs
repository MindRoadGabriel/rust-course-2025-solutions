use std::collections::HashMap;

// Part 1
struct Wrapper<'storage> {
    storage: &'storage str,
}
fn extract_from_wrapper<'a>(wrapper: &'_ Wrapper<'a>) -> &'a str {
    wrapper.storage
}

// Part 2
fn get_value<'a>(map: &HashMap<&str, &'a str>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(value) => Some(value),
        None => None,
    }
}
// Part 3
fn insert_value<'map_key, 'key, 'map_value, 'value>(map: &mut HashMap<&'map_key str, &'map_value str>, key: &'key str, value: &'value str)
    where 'key: 'map_key, 'value: 'map_value {
    map.insert(key, value);
}

// Part 4
struct BigObject<'countries, 'cities, 'map, 'inner, 'inner_string> {
    country_name_to_cities: HashMap<&'countries str, Vec<&'cities str>>,
    int_to_string: &'map HashMap<i32, String>,
    inner_object: &'inner Inner<'inner_string>,
}
struct Inner<'inner_string> {
    inner_number: u32,
    inner_string: &'inner_string str,
}
fn divide_big_object<'object, 'countries, 'cities, 'map, 'inner, 'inner_string>(
    object: &'object BigObject<'countries, 'cities, 'map, 'inner, 'inner_string>,
) -> Option<(
    &'object &'countries str,
    &'object &'cities str,
    &'map i32,
    &'map str,
    &'inner u32,
    &'inner_string str,
)> {
    if let Some((country, cities)) = object.country_name_to_cities.iter().next() {
        if let Some(first_city) = cities.first() {
            if let Some((id_int, mapped_string)) = object.int_to_string.iter().next() {
                return Some((
                    country,
                    first_city,
                    id_int,
                    mapped_string,
                    &object.inner_object.inner_number,
                    object.inner_object.inner_string,
                ));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::lifetimes_solution::*;
    use std::collections::HashMap;

    // Part 1
    #[test]
    fn test_extract_from_wrapper() {
        let string = "value".to_string();
        let wrapper = Wrapper {
            storage: &string,
        };
        let extracted = extract_from_wrapper(&wrapper);
        drop(wrapper);
        assert_eq!(extracted, "value");
    }

    // Part 2
    #[test]
    fn test_get_value() {
        let key = "key_string".to_string();
        let value = "value_string".to_string();
        let mut map = HashMap::<&str, &str>::new();
        map.insert(&key, &value);
        let fetching_key = "key_string".to_string();
        let returned_value = get_value(&map, &fetching_key);
        drop(map);
        drop(key);
        drop(fetching_key);
        assert_eq!(returned_value, Some("value_string"));
    }

    // Part 3
    #[test]
    fn test_insert_value() {
        let key = "key_string".to_string();
        let value = "value_string".to_string();
        let mut map = HashMap::<&str, &str>::new();
        insert_value(&mut map, &key, &value);
        let fetching_key = "key_string".to_string();
        let returned_value = map.get(&fetching_key as &str);
        assert_eq!(returned_value.unwrap(), &"value_string");
        drop(map);
        assert_eq!(key, "key_string");
        assert_eq!(value, "value_string");
    }

    // Part 4
    #[test]
    fn test_annoying_object() {
        let inner_string = "inner".to_string();
        let inner = Inner {
            inner_number: 34,
            inner_string: &inner_string,
        };
        let country_name = "USA".to_string();
        let city_name = "New York".to_string();
        let city_name_2 = "San Fransisco".to_string();
        let city_names: Vec<&str> = vec![&city_name, &city_name_2];
        let mut country_name_to_cities: HashMap<&str, Vec<&str>> = HashMap::new();
        country_name_to_cities.insert(&country_name, city_names);
        let mut int_to_string: HashMap<i32, String> = HashMap::new();
        int_to_string.insert(14, "mapped_string".to_string());
        let big_object = BigObject {
            country_name_to_cities,
            int_to_string: &int_to_string,
            inner_object: &inner,
        };
        if let Some((
            country_ref,
            city_ref,
            id_int_ref,
            mapped_string_ref,
            inner_number_ref,
            inner_string_ref,
        )) = divide_big_object(&big_object)
        {
            assert_eq!(*country_ref, "USA");
            assert_eq!(*city_ref, "New York");
            assert_eq!(mapped_string_ref, "mapped_string");
            assert_eq!(*id_int_ref, 14);
            drop(int_to_string);
            assert_eq!(*inner_number_ref, 34);
            drop(inner);
            assert_eq!(inner_string_ref, "inner");
            drop(inner_string);
        } else {
            panic!("Didn't divide object");
        }
    }
}
