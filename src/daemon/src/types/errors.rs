use std::fmt::{Debug, Display};

#[derive(Debug)]
pub(crate) struct WsConfigError {
    pub msg: String
}

impl WsConfigError {
    pub fn new(msg: String) -> Self {
        WsConfigError { msg }
    }
}

#[derive(Debug)]
pub(crate) struct MonitorManagerError {
    pub msg: String
}

impl MonitorManagerError {
    pub fn new(msg: String) -> Self {
        MonitorManagerError { msg }
    }
}

#[derive(Debug)]
pub(crate) struct SocketError {
    pub msg: String
}

impl SocketError {
    pub fn new(msg: String) -> Self {
        SocketError { msg }
    }
}

#[derive(Debug)]
pub(crate) enum ClientError {
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

pub(crate) struct HandlerError {
    pub log: Option<String>,
    pub client: Option<String>
}

impl HandlerError {
    pub fn user<T: Into<String>>(msg: T) -> Self {
        Self { log: None, client: Some(msg.into()) }
    }

    pub fn log<T: Into<String>>(msg: T) -> Self {
        Self { log: Some(msg.into()), client: None }
    }

    pub fn both<L: Into<String>, U: Into<String>>(log: L, client: U) -> Self {
        Self { log: Some(log.into()), client: Some(client.into()) }
    }
}
