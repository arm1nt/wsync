use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub(crate) struct SocketError {
    pub msg: String
}

impl SocketError {
    pub fn new(msg: String) -> Self {
        SocketError { msg }
    }
}
