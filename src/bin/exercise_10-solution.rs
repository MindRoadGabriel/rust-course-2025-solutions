/// Exercise 10: Tie it all together (server)
///
/// Implement a game where the server picks a random city name and sends it to
/// all connected clients and let them guess the coordinates.
/// When all clients have answered, send the answer to each of them and then
/// print out the name of client that made the best guess to the console.

use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use apricity::Coordinate;
use rand::prelude::*;
use rustdemo::{City, load_cities};
use rustdemo::protocol::*;

// enable windows feature "telnet client"
// then run
// telnet localhost 12345

pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, ClientMessage),
    Disconnect(u32),
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_name = "Example implementation server".to_string();
    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {

        let cities = load_cities().unwrap();
        let mut sockets = HashMap::new();
        let mut names = HashMap::new();
        let mut guesses = HashMap::<u32, Coordinate>::new();
        let mut actual_location = Coordinate::new(0.0, 0.0);
        let mut city_name = String::new();
        let mut rng = thread_rng();
        new_round(&cities, &mut sockets, &mut guesses, &mut actual_location, &mut city_name, &mut rng);
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, mut socket) => {
                    sockets.insert(socket_id, socket);
                }
                SocketEvent::Message(socket_id, message) => {
                    match message {
                        ClientMessage::Hello { name } => {
                            println!(r#""{}" says hello"#, name);
                            if let Some(mut stream) = sockets.get(&socket_id) {
                                names.entry(socket_id).or_insert(name);
                                let welcome = bincode::serialize(&ServerMessage::Welcome { server_name: server_name.clone() }).unwrap();
                                stream.write(&welcome).unwrap();
                                let new_round = bincode::serialize(&ServerMessage::NewRound { city_name: city_name.clone() }).unwrap();
                                stream.write(&new_round).unwrap();
                            }
                        }
                        ClientMessage::Guess(coordinate) => {
                            println!(r#"Got a guess from {} at {:?}"#, socket_id, coordinate);
                            guesses.entry(socket_id).or_insert(coordinate);
                        }
                    }
                }
                SocketEvent::Disconnect(socket_id) => {
                    guesses.remove(&socket_id);
                    sockets.remove(&socket_id);
                    names.remove(&socket_id);
                }
            }
            if sockets.len() > 0 && guesses.len() == sockets.len() {
                println!("End of round, had {} guesses and {} sockets", guesses.len(), sockets.len());
                let best_guess = guesses.iter().min_by_key(|(current_id, current_coordinate)| {
                    (actual_location.great_circle_distance(**current_coordinate) * 1000.0) as u64
                });
                if let Some((best_id, _))=  best_guess {
                    if let Some(name) = names.get(best_id) {
                        println!("{} was closest!", name);
                    }
                }
                let round_results = bincode::serialize(&ServerMessage::RoundResults { actual_location }).unwrap();
                for (_, mut stream) in sockets.iter() {
                    stream.write(&round_results).unwrap();
                }
                new_round(&cities, &mut sockets, &mut guesses, &mut actual_location, &mut city_name, &mut rng);
                println!(r#"Next round, new city is {}"#, city_name);
            }
        }
    });
    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    let mut socket_counter = 0;
    println!("Server online");
    for socket in listener.incoming() {
        let socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            }
        };
        let socket_id = socket_counter;
        socket_counter += 1;

        let socket_clone = socket.try_clone().unwrap();

        let tx = tx.clone();
        std::thread::spawn(move || {
            tx.send(SocketEvent::Connect(socket_id, socket_clone)).unwrap();
            //socket.write(": ".to_string().as_bytes()).unwrap();
            while let Ok(message) = bincode::deserialize_from::<&TcpStream, ClientMessage>(&socket) {
                tx.send(SocketEvent::Message(socket_id, message)).unwrap();
            }
            tx.send(SocketEvent::Disconnect(socket_id)).unwrap();
        });
    }
    Ok(())
}

fn new_round(cities: &Vec<City>, sockets: &mut HashMap<u32, TcpStream>, guesses: &mut HashMap<u32, Coordinate>, actual_location: &mut Coordinate, city_name: &mut String, rng: &mut ThreadRng) {
    let new_city = cities.choose(rng).unwrap();
    *city_name = new_city.fields.name.to_string();
    *actual_location = new_city.geometry.coordinates;
    guesses.clear();
    let new_round = bincode::serialize(&ServerMessage::NewRound { city_name: city_name.clone() }).unwrap();
    for (socket_id, mut stream) in sockets.iter() {
        println!("Sending new round to {}", socket_id);
        stream.write(&new_round).unwrap();
    }
}