use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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
                content: String::from("You are a helpful assistant.") 
            },
            Message::User { 
                content: String::from("Hello, how are you?") 
            },
        ]
    }
}
