/// Exercise 7: Refactoring the server to use channels instead of RwLock.
///
/// a) Duplicate exercise_7.rs to exercise_8.rs.
/// b) Make a central thread that receives messages when a socket connects, disconnects or receives data from its remote client.
/// c) Move the vector of sockets to a central thread, and have it add and remove sockets based on the messages.
/// d) Have the central thread forward text from any sockets to all other sockets.
/// e) Use the main thread for listening for new connections and creating client threads.
///
/// Useful snippets:
///    let (sender, receiver) = std::sync::mpsc::channel::<SocketEvent>();
///    sender.send(SocketEvent::Connect(socket_id, socket_clone))
///


use std::{collections::HashMap, net::TcpStream, sync::mpsc::{Receiver, Sender}};
use std::io::{ Read, Write };
use codepage_437::{BorrowFromCp437, CP437_CONTROL};

pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, String),
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
            SocketEvent::Message(sender_id, message) => {
                println!(r#"Received message "{message}" from socket_id {sender_id},"#);
                for (socket_id, socket) in &mut sockets {
                    if *socket_id != sender_id {
                        socket.write(&message.clone().into_bytes()).unwrap();
                        println!("   ...parroting to socket_id {socket_id},", );
                    }
                }
            },
            SocketEvent::Disconnect(socket_id) => {
                sockets.remove(&socket_id);
                println!("Connection lost. socket_id = {socket_id}");
            },
        }
    }
}

fn connection_handler(socket_id: u32, mut socket: TcpStream, sender: Sender<SocketEvent>) {
    let socket2 = socket.try_clone().unwrap();
    if let Err(error) = sender.send(SocketEvent::Connect(socket_id, socket2)) {
        eprintln!("{:?}", error);
        let _ = socket.shutdown(std::net::Shutdown::Both);
        return;
    }

    let mut buffer = [0; 1024];
    while let Ok(len) = socket.read(&mut buffer) {
        if len == 0 {
            break;
        }
        let message: String = BorrowFromCp437::borrow_from_cp437(&buffer[0..len], &CP437_CONTROL);
        match sender.send(SocketEvent::Message(socket_id, message)) {
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

