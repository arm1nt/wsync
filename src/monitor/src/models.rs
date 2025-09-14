use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WorkspaceInfo {
    pub(crate) name: String,
    pub(crate) local_path: PathBuf,
    pub(crate) remote_workspaces: Vec<RemoteWorkspace>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct RemoteWorkspace {
    pub(crate) name: String,
    pub(crate) remote_path: PathBuf,
    pub(crate) connection_info: ConnectionInfo
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum ConnectionInfo {
    Ssh {
        host: String,
        port: Option<u16>,
        username: Option<String>,
        identity_file: Option<PathBuf>
    },
    HostAlias {
        host_alias: String
    },
    RsyncDaemon {
        host: String,
        port: Option<u16>,
        username: Option<String>
    }
}

pub(crate) struct Error {
    pub(crate) msg: String
}

impl Error {
    pub fn new<T: AsRef<str>>(msg: T) -> Self {
        Error { msg: msg.as_ref().to_string() }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::new(format!("I/O: {value}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
