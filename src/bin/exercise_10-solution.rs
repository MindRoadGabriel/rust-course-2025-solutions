/// Exercise 10:

use apricity::{Coordinate, Point, gui::*};
use rustdemo::{helpers::exercise_10::draw_geo::*, protocol::*};
use std::{net::TcpStream, sync::mpsc, thread, time::Duration};

const SERVER_IP: &str = "127.0.0.1";
const SERVER_PORT: u16 = 12345;

const WINDOW_WIDTH: u32 = 1500;
const WINDOW_HEIGHT: u32 = 750;

const FONT_SIZE: f32 = 70.0;
const FONT_COLOR: [u8;3] = [0xFF, 0x00, 0x00];


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DisplayState {
    WaitForGuess,
    WaitForServer {
        guess: Coordinate,
    },
    WaitForContinue {
        guess: Coordinate,
        actual: Coordinate,
    },
}

pub struct GameState {
    pub current_city: String,
    pub display: DisplayState,
    pub sender: mpsc::Sender<ClientMessage>,
}

impl GameState {
    fn handle_click(&mut self, click_point: Point, window_width: u32, window_height: u32) {
        match self.display {
            DisplayState::WaitForGuess => {
                let coordinate = click_point.coordinate(
                    window_width as f64,
                    window_height as f64,
                );

                self.sender.send(
                    ClientMessage::Guess(coordinate)
                ).unwrap();
                self.display = DisplayState::WaitForServer {
                    guess: coordinate,
                };
            },
            DisplayState::WaitForServer { .. } => {},
            DisplayState::WaitForContinue { .. } => {
                self.display = DisplayState::WaitForGuess;
            },
        }
    }
}

fn load_font() -> Font<'static> {
    Font::try_from_bytes(ttf_noto_sans::REGULAR).unwrap()
}

fn connect_to_server() -> (mpsc::Sender<ClientMessage>, mpsc::Receiver<ServerMessage>) {
    let (client_msg_sender, client_msg_receiver) = mpsc::channel();
    let (server_msg_sender, server_msg_receiver) = mpsc::channel();

    let socket = TcpStream::connect((SERVER_IP, SERVER_PORT)).unwrap();

    let socket2 = socket.try_clone().unwrap();
    std::thread::spawn(move || {
        loop {
            let response: ServerMessage = bincode::deserialize_from(&socket2).unwrap();
            server_msg_sender.send(response).unwrap()
        }
    });

    std::thread::spawn(move || {
        for message in client_msg_receiver {
            bincode::serialize_into(&socket, &message).unwrap();

        }
    });

    (client_msg_sender, server_msg_receiver)
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let world_map = create_world_map(WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let window = SimpleWindow::new(WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let font = load_font();

    let (sender, receiver) = connect_to_server();


    sender.send(ClientMessage::Hello { name: "Ref. Client".to_string() })?;

    let current_city = loop {
        if let ServerMessage::Welcome { server_name } = receiver.recv()? {
            println!("Connected to {}", server_name);
            if let ServerMessage::NewRound { city_name } = receiver.recv()? {
                break city_name.clone();
            }
        }

        print!("Got an unexpected response from server.");
        thread::sleep(Duration::from_millis(1500));
        println!(" ...trying again.");
        sender.send(ClientMessage::Hello { name: "Ref. Client".to_string() })?;
    };

    let game_state = GameState {
        current_city: current_city.clone(),
        display: DisplayState::WaitForGuess,
        sender
    };

    let mut banner = SimpleImage::create_text_image(&font, &current_city, FONT_SIZE, FONT_COLOR)?;

    let mut last_display_state = game_state.display.clone();
    window.run(game_state, |window, game_state, events| {
        // Handle server messages
        while let Ok(message) = receiver.try_recv() {
            match dbg!(message) {
                ServerMessage::Welcome {..} => {},
                ServerMessage::NewRound { city_name } => {
                    game_state.current_city = city_name;
                }
                ServerMessage::RoundResults { actual_location: actual } => {
                    match game_state.display {
                        DisplayState::WaitForServer { guess } => {
                            game_state.display = DisplayState::WaitForContinue {
                                guess,
                                actual,
                            };
                        },
                        error => {
                            Err(format!("Unexpected state {:#?}", error.clone()).to_string())?
                        }
                    }
                },
            }
        }

        // Read input
        for event in events {
            if let Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } = event {
                let mouse_clicked = mouse_btn == MouseButton::Left && clicks == 1;
                if mouse_clicked {
                    let click_point = Point::new(x as f64, y as f64);
                    game_state.handle_click(click_point, WINDOW_WIDTH, WINDOW_HEIGHT);
                }
            }
        }

        // Handle game state
        let current_display_state = &game_state.display;
        if *current_display_state != last_display_state {
            let screen_text = match *current_display_state {
                DisplayState::WaitForServer { guess: _ } => "Waiting for other players".to_string(),
                DisplayState::WaitForContinue { guess, actual } => {
                    let distance_km = (guess.great_circle_distance(actual)/1000.0) as u32;
                    format!("You were {}km off", distance_km)
                },
                DisplayState::WaitForGuess => format!("Click on {}", game_state.current_city),
            };
            banner = SimpleImage::create_text_image(&font, &screen_text, FONT_SIZE, FONT_COLOR)?;
        }

        // Draw
        let half_width = (WINDOW_WIDTH/2) as i32;
        draw_image(window, &world_map, (0,0), Alignment::Left);
        draw_image(window, &banner, (half_width, 25), Alignment::Center);
        match *current_display_state {
            DisplayState::WaitForServer { guess } => {
                let p = guess.screen(window.width() as f64, window.height() as f64);
                window.stroke_circle(p.x,
                                     p.y,
                                     5.0,
                                     2.0,
                                     [ 0xFF, 0, 0, 0xFF ])?;
            },
            DisplayState::WaitForContinue { guess, actual } => {
                let guess_point = guess.screen(window.width() as f64, window.height() as f64);
                let actual_point = actual.screen(window.width() as f64, window.height() as f64);

                window.stroke_circle(guess_point.x,
                                     guess_point.y,
                                     5.0,
                                     2.0,
                                     [ 0xFF, 0, 0, 0xFF ])?;

                window.stroke_circle(actual_point.x,
                                     actual_point.y,
                                     5.0,
                                     3.0,
                                     [ 0xFF, 0xFF, 0xFF, 0xFF ])?;
            },
            DisplayState::WaitForGuess => {},
        };

        last_display_state = current_display_state.clone();
        Ok(())
    })?;

    Ok(())
}