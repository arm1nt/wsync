use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub(crate) struct MonitorManagerError {
    pub msg: String
}

#[derive(Debug)]
pub(crate) struct SocketError {
    pub msg: String
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
