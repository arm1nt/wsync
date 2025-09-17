pub(crate) mod socket;
pub(crate) mod models;

#[derive(Debug)]
pub(crate) struct Error {
    pub(crate) msg: String
}

impl Error {
    pub(crate) fn new(msg: String) -> Self {
        Error { msg }
    }
}
