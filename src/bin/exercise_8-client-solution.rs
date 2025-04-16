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

use std::net::TcpStream;
use rustdemo::protocol::{ClientMessage, ServerMessage};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpStream::connect(("127.0.0.1", 12345))?;
    let message_to_server = "Echo!".to_string();
    let client_message = ClientMessage::Hello { name: message_to_server };
    let buffer = bincode::serialize_into::<&TcpStream, ClientMessage>(&socket, &client_message)?;
    println!(r#"Sending "{:#?}" to server."#, buffer.clone());

    //socket.write(&buffer)?;
    println!("Message sent.");
    let server_message = bincode::deserialize_from::<&TcpStream, ServerMessage>(&socket)?;
    print!("Server responded: ");
    match server_message {
        ServerMessage::Welcome { server_name } => {
            println!(r#"Welcome sever_name="{server_name}""#);
        },
        ServerMessage::NewRound { city_name } => {
            println!(r#"NewRound city_name="{city_name}]""#);
        }
        ServerMessage::RoundResults { actual_location } => {
            println!("RoundResults actual_location={} lat, {} lon", actual_location.lat(), actual_location.lon());
        }
    }
    Ok(())
}



