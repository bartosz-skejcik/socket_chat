pub mod events;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    time::SystemTime,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub datetime: SystemTime,
    pub client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub is_server: bool,
    pub addr: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Server {
    pub messages: Vec<Message>,
    pub connections: Vec<Arc<Mutex<TcpStream>>>,
}

impl Server {
    pub fn new_client(&mut self, client: Client) {
        self.messages.push(Message {
            content: format!("{} has joined the chat", client.name),
            datetime: SystemTime::now(),
            client,
        });
    }

    pub fn get_client(&self, addr: &String) -> Option<&Client> {
        let client = self.messages.iter().find(|m| m.client.addr == *addr);

        match client {
            Some(client) => Some(&client.client),
            None => None,
        }
    }

    pub fn new_message(&mut self, message: String, client: &Client) {
        let message = Message {
            content: message,
            datetime: SystemTime::now(),
            client: client.clone(),
        };

        println!("{}: {}", message.client.name, message.content);

        self.messages.push(message);
    }

    pub fn broadcast(&mut self, message: &str) -> Result<(), Error> {
        for connection in &self.connections {
            let mut stream = connection.lock().unwrap();
            stream.write_all(message.as_bytes())?;
            stream.flush()?;
        }

        Ok(())
    }

    pub fn add_connection(&mut self, stream: TcpStream) {
        self.connections.push(Arc::new(Mutex::new(stream)));
    }
}

impl Client {
    pub fn new_server(addr: String) -> Self {
        Self {
            is_server: true,
            addr,
            name: "Server".to_string(),
        }
    }

    pub fn new(addr: String, name: String) -> Self {
        Self {
            is_server: false,
            addr,
            name,
        }
    }
}
