mod utils;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Error;
use utils::{Client, Server};

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:42069").unwrap();

    let mut server = Server {
        messages: Vec::new(),
    };

    for stream in listener.incoming() {
        let stream = stream?;

        handle_connection(stream, &mut server)?;
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, server: &mut Server) -> Result<(), Error> {
    let addr = stream.peer_addr()?.to_string();
    let addr = addr.split(":").collect::<Vec<&str>>()[0].to_string();
    let mut buffer = [0; 1024];

    println!("Connection from: {addr}");

    let client = identify_client(&mut stream, server, addr)?;

    let bytes_read = stream.read(&mut buffer)?;

    let message = String::from_utf8_lossy(&buffer[..bytes_read]);

    if message != "" {
        server.new_message(message.to_string(), &client);
    }

    stream.flush()?;

    Ok(())
}

fn identify_client(
    stream: &mut TcpStream,
    server: &mut Server,
    addr: String,
) -> Result<Client, Error> {
    let client = server.get_client(&addr);

    match client {
        Some(client) => {
            stream.flush()?;

            Ok(client.clone())
        }
        None => {
            // send a message to the client with the question to identify themselves
            stream.write_all(b"identify")?;

            // read the client's response
            let mut buffer = [0; 1024];
            let bytes_read = stream.read(&mut buffer)?;

            let name = String::from_utf8_lossy(&buffer[..bytes_read]);

            // update the client's name
            let client: Client = Client::new(addr, name.to_string());

            server.new_client(client.clone());

            stream.flush()?;

            Ok(client)
        }
    }
}
