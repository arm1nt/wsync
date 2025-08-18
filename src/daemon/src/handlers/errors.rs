use std::fmt::Display;

pub(super) struct HandlerError {
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
