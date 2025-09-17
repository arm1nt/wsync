use std::fmt::{Display, Formatter};

pub enum Error {
    Io(String),
    Environment(String),
    MalformedConfigFile(String),
    Initialization(String)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("{value}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(msg) => write!(f, "[IO_Error] {msg}"),
            Error::Environment(msg) => write!(f, "[Environment_Error] {msg}"),
            Error::MalformedConfigFile(msg) => write!(f, "[Malformed_Config_File_Error] {msg}"),
            Error::Initialization(msg) => write!(f, "[Initialization_Error] {msg}")
        }
    }
}
