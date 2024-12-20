use anyhow::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

mod utils;

use utils::events::{Event, ServerMessage};
use utils::{Client, Server};

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:42069")?;

    // Wrap server in Arc<Mutex<>>
    let server = Arc::new(Mutex::new(Server {
        messages: Vec::new(),
        connections: Vec::new(), // You might need to add this to your Server struct
    }));

    println!("Server listening on port 42069");

    for stream in listener.incoming() {
        let stream = stream?;
        let server = Arc::clone(&server);

        // Spawn a new thread for each connection
        thread::spawn(move || {
            if let Err(e) = handle_connection(stream, &server) {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, server: &Arc<Mutex<Server>>) -> Result<(), Error> {
    let addr = stream.peer_addr()?.to_string();
    //let addr = addr.split(":").collect::<Vec<&str>>()[0].to_string(); // getting rid of the port
    let mut buffer = [0; 1024];

    let stream_clone = stream.try_clone()?;
    {
        let mut server = server.lock().unwrap();
        server.add_connection(stream_clone);
    }

    let client = identify_client(&mut stream, server, addr)?;

    let user_joined_event = ServerMessage::new(Event::UserJoined {
        name: client.name.clone(),
    })
    .serialize()?;

    stream.write_all(user_joined_event.as_bytes())?;
    stream.flush()?;

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                println!("There was an error while reading from the stream: {e}");
                break;
            }
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read]);

        if !message.is_empty() {
            let message_event = ServerMessage::new(Event::Message {
                content: message.to_string(),
                sender: client.name.clone(),
            })
            .serialize()?;

            stream.write_all(message_event.as_bytes())?;

            let mut server = server.lock().unwrap();
            server.new_message(message.to_string(), &client);
            server.broadcast(&message_event)?;
        } else {
            println!("{} just left!", client.name);
        }

        // clear the buffer for next read
        buffer = [0; 1024];
    }

    stream.flush()?;
    Ok(())
}

fn identify_client(
    stream: &mut TcpStream,
    server: &Arc<Mutex<Server>>,
    addr: String,
) -> Result<Client, Error> {
    let client_option = server.lock().unwrap().get_client(&addr).cloned();

    match client_option {
        Some(client) => {
            let ready_event = ServerMessage::new(Event::Ready {
                client_name: client.name.clone(),
            })
            .serialize()?;

            stream.write_all(ready_event.as_bytes())?;
            stream.flush()?;

            Ok(client.clone())
        }
        None => {
            let identify_event = ServerMessage::new(Event::Identify {
                message: "Please identify yourself".to_string(),
            })
            .serialize()?;
            stream.write_all(identify_event.as_bytes())?;
            stream.flush()?;

            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer)?;

            let name = String::from_utf8_lossy(&buffer[..bytes_read]);
            let client = Client::new(addr, name.to_string());

            let ready_event = ServerMessage::new(Event::Ready {
                client_name: client.name.clone(),
            })
            .serialize()?;

            stream.write_all(ready_event.as_bytes())?;
            stream.flush()?;

            {
                let mut server = server.lock().unwrap();
                server.new_client(client.clone());
            }

            Ok(client)
        }
    }
}
