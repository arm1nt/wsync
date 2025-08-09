use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Exhaustive enumeration of all commands understood and accepted by the wsync daemon.
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    WorkspaceInfo,
    ListWorkspaces,
    ListWorkspaceInfo,
    AddWorkspace,
    RemoveWorkspace,
    AttachRemoteWorkspace,
    DetachRemoteWorkspace
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceInfoRequest {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddWorkspaceRequest {
    pub name: String,
    pub path: PathBuf
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveWorkspaceRequest {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttachRemoteWorkspace {
    pub local_workspace_name: String,
    pub remote_workspace_name: String,
    pub remote_workspace_path: PathBuf,
    pub connection_info: ConnectionInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConnectionInfo {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DetachRemoteWorkspace {
    pub local_workspace_name: String,
    pub remote_workspace_name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseCode {
    SUCCESS,
    ERROR
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub code: ResponseCode,
    pub complete: bool,
    pub msg: Option<String>
}
