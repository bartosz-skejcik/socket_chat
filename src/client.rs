use anyhow::Error;
use std::io::Read;
use std::{io::Write, net::TcpStream};

fn prompt(label: &str) -> String {
    println!("{}", label);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

fn main() -> Result<(), Error> {
    let mut stream = TcpStream::connect("0.0.0.0:42069")?;

    println!("Connected to the server");

    handle_identify(&mut stream)?;

    loop {
        let message = prompt("Enter a message to send to the server");

        match stream.write_all(message.as_bytes()) {
            Ok(_) => println!("{message}"),
            Err(e) => {
                println!("There was an error while sending the message: {e}");
                break;
            }
        };
    }

    Ok(())
}

fn handle_identify(stream: &mut TcpStream) -> Result<(), Error> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    let message = String::from_utf8_lossy(&buffer[..bytes_read]);

    if message == "identify" {
        let name = prompt("Enter your name");

        stream.write_all(name.as_bytes())?;
    }

    Ok(())
}
