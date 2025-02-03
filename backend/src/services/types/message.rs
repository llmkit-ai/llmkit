use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "role")]
pub enum Message {
    System { content: String },
    User { content: String },
    Assistant { content: String },
}
