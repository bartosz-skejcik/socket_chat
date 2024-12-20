use std::time::SystemTime;

#[derive(Debug)]
pub struct Message {
    pub content: String,
    pub datetime: SystemTime,
    pub client: Client,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub is_server: bool,
    pub addr: String,
    pub name: String,
}

#[derive(Debug)]
pub struct Server {
    pub messages: Vec<Message>,
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
