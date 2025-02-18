/// Exercise 8:
/// Simple client and message serialization:
/// Use the protocol.rs file and add bincode to your project, and send a
/// ClientMessage::Hello to your server. Decode incoming messages from the
/// server, and do a print if it's a ServerMessage::Welcome.
/// Modify the server to decode the incoming ClientMessage and send a
/// ServerMessage::Welcome back if the incoming message was ClientMessage::Hello
/// Useful snippets:
///    let outgoing_message = bincode::serialize::<ClientMessage>(&client_message);
///    let incoming_message = bincode::deserialize_from::<&TcpStream, ServerMessage>(&socket);

use std::{collections::HashMap, net::TcpStream, sync::mpsc::{Receiver, Sender}};
use rustdemo::protocol::{ClientMessage, ServerMessage};

pub enum SocketEvent {
    Connect(u32, TcpStream),
    ClientMessage(u32, ClientMessage),
    Disconnect(u32),
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
    let mut sockets = HashMap::new();
    for event in receiver {
        match event {
            SocketEvent::Connect(socket_id, socket) => {
                sockets.insert(socket_id.clone(), socket);
                println!("New connection received. socket_id = {socket_id}");
            },
            SocketEvent::ClientMessage(sender_id, message) => {
                let response = match message {
                    ClientMessage::Guess(guess) => {
                        println!("Received a Guess from socket_id {sender_id}. Let's just pretend they got it right.");
                        ServerMessage::RoundResults { actual_location: guess }
                    },
                    ClientMessage::Hello { name } => {
                        println!(r#"Received a Hello from "{name}" from socket_id {sender_id},"#);
                        ServerMessage::Welcome { server_name: "Exercise 8 server".to_string() }
                    }
                };

                let socket = sockets.get(&sender_id).unwrap();
                if let Err(error) = bincode::serialize_into(socket, &response) {
                    println!("Couldn't send message back to client. {:#?}", error);
                }
            },
            SocketEvent::Disconnect(socket_id) => {
                sockets.remove(&socket_id);
                println!("Connection lost. socket_id = {socket_id}");
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

    while let Ok(message) = bincode::deserialize_from::<&TcpStream, ClientMessage>(&socket) {
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

