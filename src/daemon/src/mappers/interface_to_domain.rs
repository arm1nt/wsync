use daemon_interface::request::{AddWorkspaceRequest, AttachRemoteWorkspaceRequest};
use crate::domain::models::{ConnectionInfo, RemoteWorkspace, WorkspaceInformation};

impl From<AddWorkspaceRequest> for WorkspaceInformation {
    fn from(value: AddWorkspaceRequest) -> Self {
        Self {
            name: value.name,
            local_path: value.path,
            remote_workspaces: vec![]
        }
    }
}

impl From<AttachRemoteWorkspaceRequest> for RemoteWorkspace {
    fn from(value: AttachRemoteWorkspaceRequest) -> Self {
        Self {
            name: value.remote_workspace_name,
            remote_path: value.remote_workspace_path,
            connection_info: ConnectionInfo::from(value.connection_info)
        }
    }
}

impl From<daemon_interface::ConnectionInfo> for ConnectionInfo {
    fn from(value: daemon_interface::ConnectionInfo) -> Self {
        match value {
            daemon_interface::ConnectionInfo::Ssh {
                host,
                port,
                username,
                identity_file
            } => {
                ConnectionInfo::Ssh {
                    host,
                    port,
                    username,
                    identity_file
                }
            },
            daemon_interface::ConnectionInfo::HostAlias { host_alias } => {
                ConnectionInfo::HostAlias { host_alias }
            },
            daemon_interface::ConnectionInfo::RsyncDaemon {
                host,
                port,
                username
            } => {
                ConnectionInfo::RsyncDaemon {
                    host,
                    port,
                    username
                }
            }
        }
    }
}
