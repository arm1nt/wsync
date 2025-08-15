pub mod request;
pub mod response;
pub mod impls;

use std::fmt::Debug;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/* Common structs */

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceOverview {
    pub name: String,
    pub path: PathBuf,
    pub nr_of_remote_workspaces: usize
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub nr_of_remote_workspaces: usize,
    pub remote_workspaces: Vec<RemoteWorkspace>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteWorkspace {
    pub name: String,
    pub path: PathBuf,
    pub connection_info: ConnectionInfo
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConnectionInfo {
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
