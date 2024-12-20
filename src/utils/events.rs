use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Event {
    Message { content: String, sender: String },
    Identify { message: String },
    Ready { client_name: String },
    UserJoined { name: String },
    UserLeft { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMessage {
    pub event: Event,
}

impl ServerMessage {
    pub fn new(event: Event) -> Self {
        Self { event }
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn deserialize(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
