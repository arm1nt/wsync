use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

/// Exhaustive enumeration of all commands understood and accepted by the wsync daemon.
#[derive(Serialize, Deserialize, Debug, EnumString, AsRefStr)]
pub enum Command {
    #[strum(serialize="workspace_info")]
    WorkspaceInfo,
    #[strum(serialize="list_workspaces")]
    ListWorkspaces,
    #[strum(serialize="list_workspace_info")]
    ListWorkspaceInfo,
    #[strum(serialize="add_workspace")]
    AddWorkspace,
    #[strum(serialize="remove_workspace")]
    RemoveWorkspace,
    #[strum(serialize="attach_remote_workspace")]
    AttachRemoteWorkspace,
    #[strum(serialize="detach_remote_workspace")]
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

impl Response {

    pub fn success(msg: Option<String>) -> Self {
        Response { code: ResponseCode::SUCCESS, complete: true, msg }
    }

    pub fn error(msg: Option<String>) -> Self {
        Response { code: ResponseCode::ERROR, complete: true, msg }
    }
}
