use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use daemon_interface::AddWorkspaceRequest;

pub(crate) mod errors;
pub(crate) mod daemon_state;
pub(crate) mod socket;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WorkspaceInformation {
    pub name: String,
    pub local_path: PathBuf,
    pub remote_workspaces: Vec<RemoteWorkspace>,
}

impl From<&AddWorkspaceRequest> for WorkspaceInformation {
    fn from(value: &AddWorkspaceRequest) -> Self {
        Self { name: value.name.clone(), local_path: value.path.clone(), remote_workspaces: vec![]}
    }
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
        port: u32,
        username: String,
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
