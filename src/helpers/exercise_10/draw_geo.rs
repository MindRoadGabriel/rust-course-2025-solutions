use std::fs;
use std::error::*;
use serde::*;
use apricity::{self, gui::*, Coordinate, Point};

const COUNTRY_DATA_FILE_PATH: &str = "countries.geojson";

#[derive(Debug,Clone,Deserialize)]
#[serde(tag = "type")]
struct FeatureCollection {
    features : Vec<Feature>,
}

#[derive(Debug,Clone,Deserialize)]
#[serde(tag = "type")]
struct Feature {
    properties : Property,
    geometry : MultiPolygon,
}

#[derive(Debug,Clone,Deserialize)]
#[serde(tag = "type")]
struct Property {
    #[serde(rename = "ADMIN")]
    admin: String,
    // #[serde(rename = "ISO_A2")]
    // iso_a2: String,
    // #[serde(rename = "ISO_A3")]
    // iso_a3: String,
}

#[derive(Debug,Clone,Deserialize)]
#[serde(tag = "type")]
struct MultiPolygon {
    coordinates : Vec<Vec<Vec<Coordinate>>>
}

pub enum Alignment {
    Left,
    Center,
    Right
}

pub fn create_world_map(width: u32, height: u32) -> Result<SimpleImage, Box<dyn Error>> {
    let json_data = fs::read_to_string(COUNTRY_DATA_FILE_PATH)?;
    let all_country_data = serde_json::from_str::<FeatureCollection>(&json_data)?;

    let width_f = width as f64;
    let height_f = height as f64;

    let mut world_map = SimpleImage::new(width, height);

    // Draw background
    println!("Drawing background");
    world_map.draw_polygon(&[
        Point {x: 0.0, y: 0.0},
        Point {x: 0.0, y: height_f - 1.0},
        Point {x: width_f - 1.0, y: height_f - 1.0},
        Point {x: width_f - 1.0, y: 0.0}
    ], [0xFF, 0x00, 0x00, 0xC0]);

    for country_data in all_country_data.features.iter() {
        println!("Drawing {}", country_data.properties.admin);

        for multi_polygon_data in country_data.geometry.coordinates.iter() {
            for polygon_data in multi_polygon_data {
                let vertices = polygon_data.into_iter()
                    .map(|vertex| vertex.screen(width_f, height_f))
                    .collect::<Vec<_>>();
                world_map.draw_polygon(&vertices, [0x00, 0xA8, 0x00, 0xFF]);
            }
        }
    }

    Ok(world_map)
}

pub fn draw_image(window: &mut SimpleWindow, image: &SimpleImage, position: (i32, i32), alignment: Alignment) {
    let image_width = image.width();
    let image_height = image.height();
    let mut x = position.0;
    let mut y = position.1;
    match alignment {
        Alignment::Center => x = position.0 - (image_width/2) as i32,
        Alignment::Right => y = position.0 - image_width as i32,
        Alignment::Left => {}
    }
    let draw_rect = Rect::new(x, y, image_width, image_height);
    let blend = image_width < window.width() || image_height < window.height();
    if let Err(error) = window.draw_image(image, Some(draw_rect), blend) {
        println!("Couldn't draw image: {}", error);
    }
}