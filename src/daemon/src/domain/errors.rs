use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub(crate) enum WsConfigError {
    Io(String),
    Message(String)
}

#[derive(Debug)]
pub(crate) struct MonitorManagerError {
    pub msg: String
}

#[derive(Debug)]
pub(crate) struct SocketError {
    pub msg: String
}

impl Display for WsConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WsConfigError::Io(e) => write!(f, "I/O Error: {e}"),
            WsConfigError::Message(e) => write!(f, "{e}")
        }
    }
}

impl MonitorManagerError {
    pub fn new(msg: String) -> Self {
        MonitorManagerError { msg }
    }
}

impl Display for MonitorManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl SocketError {
    pub fn new(msg: String) -> Self {
        SocketError { msg }
    }
}
