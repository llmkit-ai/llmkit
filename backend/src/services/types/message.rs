use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase", tag = "role")]
pub enum Message {
    System { content: String },
    User { content: String },
    Assistant { content: String },
}

impl Message {
    pub fn defaults() -> Vec<Self> {
        vec![
            Message::System { 
                content: String::from("You are a helpful assistant that always says Hello") 
            },
            Message::User { 
                content: String::from("Hello, how are you?") 
            },
        ]
    }
}
