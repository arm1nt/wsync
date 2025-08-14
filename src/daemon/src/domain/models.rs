use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Internal models representing workspaces and their associated information. Those types will, and
/// already do, deviate from the structs defined in the daemon interface.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WorkspaceInformation {
    pub name: String,
    pub local_path: PathBuf,
    pub remote_workspaces: Vec<RemoteWorkspace>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct RemoteWorkspace {
    pub name: String,
    pub remote_path: PathBuf,
    pub connection_info: ConnectionInfo
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
