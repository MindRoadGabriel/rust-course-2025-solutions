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
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

pub fn main() -> Result<(), Box<dyn std::error::Error>>{
    let listener = std::net::TcpListener::bind(("0.0.0.0", 12345))?;
    let sockets = Arc::new(RwLock::new(Vec::new()));
    for socket in listener.incoming() {
        let mut socket = match socket {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:?}", e);
                continue;
            },
        };
        {
            let mut sockets_lock = sockets.write().unwrap();
            sockets_lock.push(socket.try_clone()?);
        }
        let sockets2 = sockets.clone();
        std::thread::spawn(move ||{
            socket.write(b"Hello!").unwrap();
            let mut buffer = [0; 1024];
            let len = socket.read(&mut buffer).unwrap();
            let s = std::str::from_utf8(&buffer[0..len]).unwrap();
            println!("{}", s);

            let sockets_lock = sockets2.read().unwrap();
            for mut client in sockets_lock.iter() {
                let _ = client.write(s.as_bytes());
            }
        });
    }
    Ok(())
}