/// Exercise 8:
/// Simple client and message serialization:
/// Use the protocol.rs file and add bincode to your project, and send a
/// ClientMessage::Hello to your server. Decode incoming messages from the
/// server, and do a print if it's a ServerMessage::Welcome.
/// Modify the server to echo messages back to the client that sent it as well.
/// Due to a quirk of bincode's encoding, ClientMessage::Hello and
/// ServerMessage::Welcome looks the same encoded, so the echoed message will work as a server message.
/// Useful snippets:
///    let outgoing_message = bincode::serialize::<ClientMessage>(&client_message);
///    let incoming_message = bincode::deserialize_from::<&TcpStream, ServerMessage>(&socket);

use std::{collections::HashMap, io::Write, net::TcpStream, sync::mpsc::{Receiver, Sender}};
use rand::prelude::*;

use apricity::Coordinate;
use rustdemo::{helpers::exercise_9::city_parser::{CityData, load_city_data}, protocol::{ClientMessage, ServerMessage}};

pub struct Client {
    socket: TcpStream,
    name: Option<String>,
    guess: Option<Coordinate>,
}

impl Client {
    pub fn new(socket: TcpStream) -> Client {
        Client {
            socket,
            name: None,
            guess: None,
        }
    }
}

impl Write for Client {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }
}

pub enum SocketEvent {
    Connect(u32, TcpStream),
    ClientMessage(u32, ClientMessage),
    Disconnect(u32),
}

struct GameHandler {
    cities: Vec<CityData>,
    clients: HashMap<u32, Client>,
    current_city: Option<CityData>,
    rng: ThreadRng
}

impl GameHandler {
    pub fn new() -> GameHandler {
        GameHandler {
            cities: load_city_data().unwrap(),
            clients: HashMap::new(),
            current_city: None,
            rng: thread_rng()
        }
    }

    pub fn add_client(&mut self, client_id: u32, socket: TcpStream) {
        self.clients.insert(client_id, Client::new(socket));
    }

    pub fn remove_client(&mut self, client_id: u32) {
        if let Some(client) = self.clients.remove(&client_id) {
            let _ = client.socket.shutdown(std::net::Shutdown::Both);
        }
    }

    pub fn handle_client_message(&mut self, client_id: u32, client_message: ClientMessage) {
        match client_message {
            ClientMessage::Hello { name } => {
                self.clients.entry(client_id).and_modify(|client| client.name = Some(name));
                self.send_message_to_client(client_id, &ServerMessage::Welcome { server_name: "ref-server".to_string() });

                // If there's an ongoing game, include the player.
                if let Some(current_city) = &self.current_city {
                    self.send_message_to_client(client_id, &ServerMessage::NewRound { city_name: current_city.name.to_string() });
                }
            }
            ClientMessage::Guess(coordinate) => {
                self.clients.entry(client_id).and_modify(|client | client.guess = Some(coordinate));
            }
        }

        // Check if all players have answered
        if let Some(current_city_data) = &self.current_city {
            println!("Checking if all players have answered.");
            let mut all_done = true;
            for (_, client) in self.clients.iter() {
                if client.name.is_some() {
                    // Let's not let a newly connected client detain the current game
                    all_done = all_done && client.guess.is_some();
                }
            }

            if all_done {
                println!("All done! Sending round results.");
                let actual_coordinates = current_city_data.coordinates;

                if let Some((winner_id, winner, dist)) = self.calculate_winner(&actual_coordinates) {
                    let winner_name = winner.name.clone().unwrap_or("[Unknown]".to_string());
                    println!(r#"Player "{}", socket_id {} won with a distance of {} km."#, winner_name, winner_id, (dist/1000.0) as u32);
                }
                else {
                    println!("No winner in this round.");
                }

                let round_results_message = ServerMessage::RoundResults { actual_location: actual_coordinates };
                for (socket_id, client) in self.clients.iter() {
                    if client.name.is_some() {
                        self.send_message_to_client(*socket_id, &round_results_message);
                    }
                }

                self.current_city = None;
            }
        }

        if self.current_city.is_none() {
            println!("No current game in progress. Starting new round.");
            let new_city_index: usize = self.rng.gen_range(0..self.cities.len());
            let new_city = self.cities.get(new_city_index).unwrap().clone();
            let city_name = new_city.name.to_string();
            let new_round_message = ServerMessage::NewRound { city_name: city_name };
            for (client_id, client) in self.clients.iter_mut() {
                if client.name.is_some() {
                    client.guess = None;
                    GameHandler::send_message(*client_id, &client, &new_round_message);
                }
            }

            self.current_city = Some(new_city);
        }

    }

    pub fn send_message_to_client(&self, client_id: u32, message: &ServerMessage) {
        let socket = &self.clients.get(&client_id).unwrap().socket;
        if let Err(error) = bincode::serialize_into(socket, &message) {
            println!("Couldn't send to  client_id {}: {:#?}", client_id, error);
            return;
        }

        let message_type = match message {
            ServerMessage::Welcome { server_name: _ } => "WELCOME".to_string(),
            ServerMessage::NewRound { city_name: _} => "NEW_ROUND".to_string(),
            ServerMessage::RoundResults { actual_location: _ } => "ROUND_RESULTS".to_string()
        };
        println!("Sending response {message_type} to socket_id {client_id}", );
    }

    fn send_message(client_id: u32, client: &Client, message: &ServerMessage) {
        let socket = &client.socket;
        if let Err(error) = bincode::serialize_into(socket, &message) {
            println!("Couldn't send to  client_id {}: {:#?}", client_id, error);
            return;
        }

        let message_type = match message {
            ServerMessage::Welcome { server_name: _ } => "WELCOME".to_string(),
            ServerMessage::NewRound { city_name: _} => "NEW_ROUND".to_string(),
            ServerMessage::RoundResults { actual_location: _ } => "ROUND_RESULTS".to_string()
        };
        println!("Sending response {message_type} to socket_id {client_id}", );
    }

    fn calculate_winner(&self, actual_coordinates: &Coordinate) -> Option::<(u32, &Client, f64)> {
        let mut closest_dist = f64::MAX;
        let mut closest_client = None;
        for (client_id, client) in self.clients.iter() {
            if let Some(guessed_coordinates) = client.guess {
                let dist = actual_coordinates.great_circle_distance(guessed_coordinates);
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_client = Some(*client_id);
                }
            }
        }

        if let Some(client_id) = closest_client {
            return Some((client_id, &self.clients[&client_id], closest_dist));
        }
        None
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || message_handler(receiver));

    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    let mut socket_counter: u32 = 0;
    for socket in listener.incoming() {
        let socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };

        let socket_id = socket_counter;
        socket_counter += 1;

        let client_sender = sender.clone();
        std::thread::spawn(move || connection_handler(socket_id, socket, client_sender));
    }

    Ok(())
}

fn message_handler(receiver: Receiver<SocketEvent>) {
    let mut game_handler = GameHandler::new();
    for event in receiver {
        match event {
            SocketEvent::Connect(socket_id, socket) => {
                game_handler.add_client(socket_id.clone(), socket);
                println!("New connection received. socket_id = {socket_id}");
            },
            SocketEvent::ClientMessage(sender_id, message) => {
                game_handler.handle_client_message(sender_id, message);


            },
            SocketEvent::Disconnect(socket_id) => {
                game_handler.remove_client(socket_id);
                println!("Connection to socket_id {socket_id} lost.");
            },
        }
    }
}

fn connection_handler(socket_id: u32, socket: TcpStream, sender: Sender<SocketEvent>) {
    let socket2 = socket.try_clone().unwrap();
    if let Err(error) = sender.send(SocketEvent::Connect(socket_id, socket2)) {
        eprintln!("{:?}", error);
        let _ = socket.shutdown(std::net::Shutdown::Both);
        return;
    }

    while let Ok(message) = bincode::deserialize_from(&socket) {

        match sender.send(SocketEvent::ClientMessage(socket_id, message)) {
            Ok(_) => {},
            Err(error) => {
                eprintln!("{:?}", error);
                break;
            },
        };
    }

    let _ = sender.send(SocketEvent::Disconnect(socket_id));
    let _ = socket.shutdown(std::net::Shutdown::Both);
}

