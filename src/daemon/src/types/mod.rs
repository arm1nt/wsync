use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WorkspaceInformation {
    pub name: String,
    pub local_path: PathBuf,
    pub remote_workspaces: Vec<RemoteWorkspace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct RemoteWorkspace {
    pub name: String,
    pub remote_path: PathBuf,
    pub connection_info: ConnectionInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum ConnectionInfo {
    Ssh {
        host: String,
        port: u32,
        username: String,
        password: Option<String>,
        identity_file: Option<PathBuf>
    },
    HostAlias {
        host_alias: String
    },
    RsyncDaemon {
        host: String,
        port: u32,
        username: String
    }
}
