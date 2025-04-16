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
use std::collections::HashMap;
use std::io::{Write, Read};
use std::net::TcpStream;

pub enum SocketEvent {
    Connect(u32, TcpStream),
    Message(u32, String),
    Disconnect(u32),
}

pub fn main() -> Result<(), Box<dyn std::error::Error>>{
    let (tx, rx) = std::sync::mpsc::channel::<SocketEvent>();
    std::thread::spawn(move || {
        let mut sockets = HashMap::new();
        for event in rx {
            match event {
                SocketEvent::Connect(socket_id, socket) => {
                    sockets.insert(socket_id, socket);
                },
                SocketEvent::Message(socket_id, message) => {
                    for (current_id, mut stream) in sockets.iter() {
                        if socket_id != *current_id {
                            stream.write(message.as_bytes()).unwrap();
                        }
                    }
                }
                SocketEvent::Disconnect(socket_id) => {
                    sockets.remove(&socket_id);
                }
            }
        }
    });
    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    let mut socket_counter = 0;
    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };
        let socket_id = socket_counter;
        socket_counter += 1;

        let socket_clone = socket.try_clone().unwrap();
        tx.send(SocketEvent::Connect(socket_id, socket_clone)).unwrap();

        let tx = tx.clone();
        std::thread::spawn(move ||{
            socket.write(": ".to_string().as_bytes()).unwrap();
            let mut buffer = [0; 1024];
            while let Ok(len) = socket.read(&mut buffer) {
                if len == 0 {
                    break;
                }
                let s = std::str::from_utf8(&buffer[0..len]).unwrap();
                println!("{}", &s);
                tx.send(SocketEvent::Message(socket_id, s.to_string())).unwrap();
            }
            tx.send(SocketEvent::Disconnect(socket_id)).unwrap();
        });
    }
    Ok(())
}