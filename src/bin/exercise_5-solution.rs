/// Exercise 5:
/// Add the library "apricity" to the project and create a window.
///
/// > cargo add apricity --git https://github.com/MindroadGabriel/apricity.git
/// > cargo add ttf-noto-sans
///
/// Use the helper functions in draw_geo to load the countries data and use the geometry inside
/// to draw a world map.
/// Use the code from exercise 4 to draw a circle on each city that is largest in its country.
/// Draw some text to the screen.
/// Print to console when click events happen.
/// Useful snippets:
///     let world_map: SimpleImage = draw_geo::create_world_map(window_width, window_height).unwrap();
///     let font: Font<'_> = Font::try_from_bytes(ttf_noto_sans::REGULAR).unwrap();
///     let screen_text: SimpleImage = SimpleWindow::create_text_image(font: &Font<'static>, "Some Text", font_size, color).unwrap();
///     window.run(|window, events| { ... });
///     draw_geo::draw_image(window, &image, position_on_screen, Alignment::Left);

use apricity::gui::*;
use rustdemo::helpers::exercise_5::{city_parser::*, draw_geo::*, };

const WINDOW_WIDTH: u32 = 1500;
const WINDOW_HEIGHT: u32 = 750;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cities = load_city_data()?;
    let world_map = create_world_map(WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let largest_cities = get_largest_city_for_each_country(cities);

    let window = SimpleWindow::new(WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let font: Font<'_> = Font::try_from_bytes(ttf_noto_sans::REGULAR).unwrap();
    let banner = SimpleImage::create_text_image(&font, "Rust Rules!", 100.0, [ 0xFF, 0xFF, 0 ])?;

    window.run((), |window, _state, events| {
        let half_width = (WINDOW_WIDTH/2) as i32;
        draw_image(window, &world_map, (0,0), Alignment::Left);
        draw_image(window, &banner, (half_width, 250), Alignment::Center);

        for city in &largest_cities {

            let point = city.coordinates.screen(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);

            window.stroke_circle(
                point.x,
                point.y,
                5.0,
                2.0,
                [ 0xFF, 0, 0, 0xFF ],
            )?;
        }

        for event in events {
            if let Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } = event {
                let mouse_clicked = mouse_btn == MouseButton::Left && clicks == 1;
                if mouse_clicked {
                    println!("Mouse clicked at ({}, {})", x, y);
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn get_largest_city_for_each_country(city_data : Vec<CityData>) -> Vec<CityData> {
    let mut result = city_data.clone();
    result.sort_by(|x, y| {
        if x.country_code == y.country_code {
            y.population.cmp(&x.population)
        }
        else {
            x.country_code.cmp(&y.country_code)
        }
    });
    result.dedup_by_key(|x| x.country_code.clone());
    result
}