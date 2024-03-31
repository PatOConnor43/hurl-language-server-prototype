use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub jsonrpc: String,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    /// Numeric ID.
    Number(i64),
    /// String ID.
    String(String),
}

#[derive(Deserialize, Serialize)]
pub struct RequestMessage {
    #[serde(flatten)]
    pub message: Message,

    pub id: Id,
    pub method: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResponseMessage<T> {
    #[serde(flatten)]
    pub message: Message,

    pub id: Id,
    pub result: T,
}

impl<T> ResponseMessage<T> {
    pub fn new(id: i64, result: T) -> Self {
        Self {
            message: Message {
                jsonrpc: "2.0".to_string(),
            },
            id: Id::Number(id),
            result,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Notification<T> {
    #[serde(flatten)]
    pub message: Message,
    pub method: String,
    pub params: T,
}

impl<T> Notification<T> {
    pub fn new(method: String, params: T) -> Self {
        Self {
            message: Message {
                jsonrpc: "2.0".to_string(),
            },
            method,
            params,
        }
    }
}
