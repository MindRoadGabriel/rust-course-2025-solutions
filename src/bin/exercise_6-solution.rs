/// Exercise 6: Simple server time!
/// 
/// a) Create an exercise_6.rs file.
/// b) For each incoming socket, add it to a thread-shared vector, then create a thread for it.
/// c) In that thread, write a hello message, then read the first message and send it to all other sockets.
/// d) If on windows, add the windows feature for telnet client. To handle windows terminal encoding, add codepage_437 to the project with "cargo add codepage_437".
/// e) Start three terminals, and run this in each of them:
/// telnet localhost 12345
/// f) Then try typing and see your text appear on the other terminals.
/// 
/// Useful snippets:
///    for socket in listener.incoming() {}
///    let mut buffer = [0; 1024];
///    let len = socket.read(&mut buffer).unwrap();
///    let s = std::str::from_utf8(&buffer[0..len]).unwrap();
///    use codepage_437::{BorrowFromCp437, CP437_CONTROL};
///    let s = String::borrow_from_cp437(&buffer[0..len], &CP437_CONTROL);

use std::{collections::HashMap, sync::{Arc, RwLock}};
use std::io::{ Read, Write };
use codepage_437::{BorrowFromCp437, CP437_CONTROL};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sockets = Arc::new(RwLock::new(HashMap::new()));

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
        {
            let mut sockets_lock = sockets.write().unwrap();
            sockets_lock.insert(socket_id, socket.try_clone()?);
            println!("New connection received. socket_id = {socket_id}");
        }

        let sockets2 = sockets.clone();
        std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            while let Ok(len) = socket.read(&mut buffer) {
                if len == 0 {
                    break;
                }
                let s = String::borrow_from_cp437(&buffer[0..len], &CP437_CONTROL);
                println!(r#"Recieved message "{s}" from socket_id {socket_id},"#);

                {
                    let mut sockets_lock = sockets2.write().unwrap();
                    for (receiver_id, socket) in sockets_lock.iter_mut() {
                        if *receiver_id != socket_id {
                            socket.write(&buffer[0..len]).unwrap();
                            println!("   ...parroting to socket_id {receiver_id},", );
                        }
                    }
                }
            }

            {
                let mut sockets_lock = sockets2.write().unwrap();
                sockets_lock.remove(&socket_id);
                println!("Connection lost. socket_id = {socket_id}");
            }
        });
    }

    Ok(())
}



