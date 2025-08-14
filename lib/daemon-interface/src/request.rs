use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use crate::ConnectionInfo;

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
pub struct CommandRequest {
    pub command: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceInfoRequest {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddWorkspaceRequest {
    pub name: String,
    pub path: PathBuf
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveWorkspaceRequest {
    pub name: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttachRemoteWorkspaceRequest {
    pub local_workspace_name: String,
    pub remote_workspace_name: String,
    pub remote_workspace_path: PathBuf,
    pub connection_info: ConnectionInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetachRemoteWorkspaceRequest {
    pub local_workspace_name: String,
    pub remote_workspace_name: String
}
