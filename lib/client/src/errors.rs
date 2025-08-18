use std::fmt::Display;

#[derive(Debug)]
pub enum ClientError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    Protocol(&'static str),
    Message(String)
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "I/O error: {e}"),
            ClientError::Serde(e) => write!(f, "JSON error: {e}"),
            ClientError::Protocol(e) => write!(f, "Protocol error: {e}"),
            ClientError::Message(e) => write!(f, "{e}"),
        }
    }
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}
