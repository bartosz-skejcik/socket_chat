use anyhow::Error;
use std::sync::mpsc;
use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
};

mod app;
mod utils;

use utils::events::{Event, ServerMessage};

fn prompt(_label: &str) -> String {
    //println!("{}", label);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

fn handle_identify(stream: &mut TcpStream) -> Result<(), Error> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let message = String::from_utf8_lossy(&buffer[..bytes_read]);

    match ServerMessage::deserialize(&message) {
        Ok(server_message) => match server_message.event {
            Event::Identify { message } => {
                let name = prompt(&message);
                stream.write_all(name.as_bytes())?;
                stream.flush()?;

                // Wait for Ready event
                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer)?;
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);

                match ServerMessage::deserialize(&message) {
                    Ok(server_message) => match server_message.event {
                        Event::Ready { client_name } => {
                            println!("Connected as {}", client_name);
                        }
                        _ => return Err(Error::msg("Unexpected server event")),
                    },
                    Err(e) => {
                        return Err(Error::msg(format!("Failed to parse server message: {}", e)))
                    }
                }
            }
            Event::Ready { client_name } => {
                println!("Connected as {}", client_name);
            }
            _ => return Err(Error::msg("Unexpected server event")),
        },
        Err(e) => return Err(Error::msg(format!("Failed to parse server message: {}", e))),
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    //let mut terminal = ratatui::init();
    //let app = App::default().run(&mut terminal)?;

    let mut stream = TcpStream::connect("0.0.0.0:42069")?;
    println!("Connected to the server");

    handle_identify(&mut stream)?;

    // Clone the stream for the listener thread
    let mut listener_stream = stream.try_clone()?;

    // Create a channel for communicating between threads
    let (tx, rx) = mpsc::channel();

    // Spawn listener thread
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match listener_stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Server disconnected");
                    break;
                }
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    match ServerMessage::deserialize(&message) {
                        Ok(server_message) => match server_message.event {
                            Event::Message { content, sender } => {
                                println!("{}: {}", sender, content);
                            }
                            Event::Ready { client_name } => {
                                println!("Connected as {}", client_name);
                            }
                            Event::UserJoined { name } => {
                                println!("{:?} has joined the chat", name);
                            }
                            Event::UserLeft { name } => {
                                println!("{:?} has left the chat", name);
                            }
                            _ => println!("Received unknown event: {:?}", server_message.event),
                        },
                        Err(e) => println!("Failed to parse server message: {}", e),
                    }
                }
                Err(e) => {
                    println!("Error reading from server: {}", e);
                    break;
                }
            }
            buffer = [0; 1024];
        }
        tx.send(()).unwrap(); // Signal main thread that we've disconnected
    });

    // Main thread handles sending messages
    loop {
        let message = prompt("Enter a message to send to the server");

        if message == "exit" {
            break;
        }

        match stream.write_all(message.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("There was an error while sending the message: {e}");
                break;
            }
        };

        // Check if listener thread has signaled disconnection
        if rx.try_recv().is_ok() {
            break;
        }
    }

    //ratatui::restore();
    //app;
    Ok(())
}
