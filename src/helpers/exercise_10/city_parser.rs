use std::fs;

use apricity::Coordinate;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct City {
    pub datasetid: String,
    pub recordid: String,
    pub fields: CityData,
    pub record_timestamp: String,
    pub geometry: CityGeometry,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CityGeometry {
    coordinates: Coordinate,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CityData {
    pub coordinates: Coordinate,
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

impl CityData {
    pub fn country_name_eng(&self) -> &str {
        match &self.cou_name_en {
            Some(x) => x,
            None => &self.country_code,
        }
    }
}

pub fn load_city_data() -> Result<Vec<CityData>, Box<dyn std::error::Error>> {
    let file_name = "cities100k.json";
    let json_data = fs::read_to_string(file_name)?;
    let city_data = serde_json::from_str::<Vec<City>>(&json_data)?;

    Ok(city_data.into_iter().map(|mut x| { x.fields.coordinates = x.geometry.coordinates; x.fields }).collect())
}