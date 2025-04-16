/// Exercise 8: Simple client and message serialization
///
/// Simple client!
/// a) Create exercise_8_client.rs
/// b) Use the protocol.rs file and add bincode to your project, and send a ClientMessage::Hello to your server.
/// c) Decode incoming messages from the server, and do a print if it's a ServerMessage::Welcome.
///
/// Server modifications
/// a) Duplicate exercise_7.rs to exercise_8_server.rs
/// b) Modify the server to decode the incoming ClientMessage
/// c) Send a ServerMessage::Welcome back if the incoming message was ClientMessage::Hello
///
/// Useful snippets:
///    bincode::serialize_into(&socket, &client_message);
///    let incoming_message = bincode::deserialize_from::<&TcpStream, ServerMessage>(&socket);

use std::io::Write;
use std::net::TcpStream;
use rustdemo::protocol::{ClientMessage, ServerMessage};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(("127.0.0.1", 12345))?;
    let msg = ClientMessage::Hello {
        name: "Gabriel".to_string()
    };
    let buffer = bincode::serialize(&msg)?;
    socket.write(&buffer)?;
    let reply: ServerMessage = bincode::deserialize_from(&socket)?;
    println!("{:?}", &reply);

    Ok(())
}


