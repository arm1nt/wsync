pub(in crate::handlers) struct Error {
    pub(in crate::handlers) log: Option<String>,
    pub(in crate::handlers) client: Option<String>
}

impl Error {
    pub(in crate::handlers) fn user<T: Into<String>>(msg: T) -> Self {
        Self { log: None, client: Some(msg.into()) }
    }

    pub(in crate::handlers) fn log<T: Into<String>>(msg: T) -> Self {
        Self { log: Some(msg.into()), client: None }
    }

    pub(in crate::handlers) fn both<L: Into<String>, U: Into<String>>(log: L, client: U) -> Self {
        Self { log: Some(log.into()), client: Some(client.into()) }
    }
}
