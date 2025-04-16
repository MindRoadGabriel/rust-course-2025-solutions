/// Exercise 11: Tie it all together. (Client)
///
/// Duplicate exercise_5.rs to exercise_11.rs.
/// Also use code from exercise_8.rs.
/// Implement the client for this game!
///
/// Have the client cycle through three states.
/// a) Displaying the city name and asking the user to guess where the city is by clicking,
/// b) asking the player to wait for the other players to guess,
/// c) and displaying the distance between the guess and the correct coordinate
/// along with circles for the clicked coordinate and the correct coordinate.
///
/// When you're done, connect to the teacher server and play with others who are done.

use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::thread;
use apricity::gui::{SimpleImage, Font, Event, Rect, MouseButton};
use apricity::{Coordinate, Point};
use rustdemo::helpers::exercise_11::draw_geo::create_world_map;
use rustdemo::protocol::{ClientMessage, ServerMessage};

fn load_font() -> Font<'static> {
    Font::try_from_bytes(ttf_noto_sans::REGULAR).unwrap()
}

enum GameState {
    Starting,
    Guessing {
        city_name: String,
    },
    Waiting {
        guess: Point,
    },
    Reviewing {
        guess: Point,
        actual: Point,
    },
}

#[derive(Default, Debug)]
struct TransitionInformation {
    next_city: Option<String>,
    next_actual_location: Option<Coordinate>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create background
    let width = 1500;
    let height = 750;
    let background_image = create_world_map(width, height)?;

    // Set up communication with the server
    let mut socket = TcpStream::connect(("127.0.0.1", 12345)).unwrap();
    let (tx, rx) = channel::<ServerMessage>();
    let mut socket_receive = socket.try_clone().unwrap();
    thread::spawn(move ||{
        let message = ClientMessage::Hello {
            name: "Gabriel".to_string()
        };
        socket_receive.write(&bincode::serialize(&message).unwrap()).unwrap();
        while let Ok(message) = bincode::deserialize_from::<&TcpStream, ServerMessage>(&socket_receive) {
            tx.send(message).unwrap();
        }
        println!("Lost connection to server");
    });

    let window = apricity::gui::SimpleWindow::new(width, height)?;
    let font = load_font();
    // This is data that persists between frames, but is modified during runtime
    // State is the current user-visible state of the game
    // Transition info is data from the server which can trigger a transition, but that we may
    // not be ready for when it arrives
    let mut state = GameState::Starting;
    let mut transition_info = TransitionInformation::default();
    let mut current_text_image = SimpleImage::create_text_image(&font, "Please wait...", 72.0, [0xFF, 0x22, 0])?;

    window.run((), |window, _, events| {
        // Render background
        window.draw_image(&background_image, None, false)?;

        // Render guess and actual, if in the correct state
        let red = [0xFF, 0, 0, 0xFF];
        let blue = [0, 0, 0xFF, 0xFF];
        match state {
            GameState::Waiting { guess } => {
                window.stroke_circle(guess.x, guess.y, 10.0, 1.0, red)?;
            }
            GameState::Reviewing { guess, actual } => {
                window.stroke_circle(guess.x, guess.y, 10.0, 1.0, red)?;
                window.stroke_circle(actual.x, actual.y, 10.0, 1.0, blue)?;
            }
            _ => {}
        }
        // Render text
        let rect = Rect::new(10, 10, current_text_image.width(), current_text_image.height());
        window.draw_image(&current_text_image, Some(rect), true)?;
        // Listen for clicks
        let mut click_location = None;
        for event in events {
            match event {
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    if let MouseButton::Left = mouse_btn {
                        click_location = Some(Point::new(x as f64, y as f64));
                    }
                }
                _ => {}
            }
        }
        // Listen for messages
        match rx.try_recv().ok() {
            Some(ServerMessage::Welcome { server_name }) => {
                println!("Server {} welcomes you", server_name);
            }
            Some(ServerMessage::NewRound { city_name }) => {
                println!("Next ciy: {}", city_name);
                transition_info.next_city = Some(city_name);
            }
            Some(ServerMessage::RoundResults { actual_location }) => {
                println!("Actual coordinate: {:?}", actual_location);
                transition_info.next_actual_location = Some(actual_location);
            }
            _ => {}
        }
        // See if any of the incoming events are enough to change state
        match (&state, click_location, &mut transition_info) {
            // At startup, wait until the server gives the first target town,
            // Then start guessing
            (GameState::Starting, _, TransitionInformation {
                next_city: city_option @ Some(_),
                ..
            })  |
            (GameState::Reviewing { .. }, Some(_), TransitionInformation {
                next_city: city_option @ Some(_),
                ..
            }) => {
                let city_name = city_option.take().unwrap();
                current_text_image = SimpleImage::create_text_image(&font, &format!("Where do you think {} is?", city_name), 72.0, [0xFF, 0x22, 0])?;
                println!("Entering Guessing");
                state = GameState::Guessing { city_name };
                transition_info = TransitionInformation::default();
            }

            // When we've clicked somewhere to guess, start waiting
            (GameState::Guessing { city_name }, Some(click), TransitionInformation { .. }) => {
                let coordinate = click.coordinate(width as f64, height as f64);
                let message = ClientMessage::Guess(coordinate);
                socket.write(&bincode::serialize(&message)?).unwrap();
                current_text_image = SimpleImage::create_text_image(&font, "Waiting for other players...", 72.0, [0xFF, 0x22, 0])?;

                println!("Entering Waiting");
                state = GameState::Waiting { guess: click };
            }

            // When the last player is done, we review how we did
            (GameState::Waiting { guess: guess_point }, _, TransitionInformation {
                next_actual_location: actual_option @ Some(_),
                ..
            }) => {
                let actual_coordinate = actual_option.take().unwrap();
                let actual_point = actual_coordinate.screen(width as f64, height as f64);
                let guess_coordinate = guess_point.coordinate(width as f64, height as f64);
                let distance = actual_coordinate.great_circle_distance(guess_coordinate);
                current_text_image = SimpleImage::create_text_image(&font, &format!("You were {} km away", distance as u64), 72.0, [0xFF, 0x22, 0])?;

                println!("Entering Reviewing");
                state = GameState::Reviewing { guess: guess_point.clone(), actual: actual_point };
                transition_info.next_actual_location = None;
            }
            // If none of these conditions are fulfilled, don't change state at all.
            _ => {}
        };
        Ok(())
    })
}

