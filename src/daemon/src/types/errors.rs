use std::fmt::{Debug, Display};

#[derive(Debug)]
pub(crate) struct WsConfigError {
    pub msg: String
}

#[derive(Debug)]
pub(crate) struct MonitorManagerError {
    pub msg: String
}

#[derive(Debug)]
pub(crate) struct SocketError {
    pub msg: String
}

impl WsConfigError {
    pub fn new(msg: String) -> Self {
        WsConfigError { msg }
    }
}

impl MonitorManagerError {
    pub fn new(msg: String) -> Self {
        MonitorManagerError { msg }
    }
}

impl SocketError {
    pub fn new(msg: String) -> Self {
        SocketError { msg }
    }
}
