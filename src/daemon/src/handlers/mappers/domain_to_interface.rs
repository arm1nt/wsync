use daemon_interface::response::{ListWorkspaceInfoResponse, ListWorkspacesResponse, WorkspaceInfoResponse};
use daemon_interface::WorkspaceInfo;
use crate::domain::models::{ConnectionInfo, RemoteWorkspace, WorkspaceInformation};

impl Into<daemon_interface::WorkspaceInfo> for WorkspaceInformation {
    fn into(self) -> WorkspaceInfo {
        let remote_workspaces: Vec<daemon_interface::RemoteWorkspace> = self.remote_workspaces
            .into_iter()
            .map(|rw| rw.into())
            .collect();

        WorkspaceInfo {
            name: self.name,
            path: self.local_path,
            nr_of_remote_workspaces: remote_workspaces.len(),
            remote_workspaces
        }
    }
}

impl Into<daemon_interface::RemoteWorkspace> for RemoteWorkspace {
    fn into(self) -> daemon_interface::RemoteWorkspace {
        daemon_interface::RemoteWorkspace {
            name: self.name,
            path: self.remote_path,
            connection_info: self.connection_info.into()
        }
    }
}

impl Into<daemon_interface::ConnectionInfo> for ConnectionInfo {
    fn into(self) -> daemon_interface::ConnectionInfo {
        match self {
            ConnectionInfo::Ssh {
                host,
                port,
                username,
                identity_file
            } => {
                daemon_interface::ConnectionInfo::Ssh {
                    host,
                    port,
                    username,
                    identity_file
                }
            },
            ConnectionInfo::HostAlias { host_alias } => {
                daemon_interface::ConnectionInfo::HostAlias { host_alias }
            },
            ConnectionInfo::RsyncDaemon {
                host,
                port,
                username
            } => {
                daemon_interface::ConnectionInfo::RsyncDaemon {
                    host,
                    port,
                    username
                }
            }
        }
    }
}

pub(crate) fn to_workspace_info_response(data: WorkspaceInformation) -> WorkspaceInfoResponse {
    let remote_workspaces: Vec<daemon_interface::RemoteWorkspace> = data.remote_workspaces
        .into_iter()
        .map(|rw| {
            daemon_interface::RemoteWorkspace {
                name: rw.name,
                path: rw.remote_path,
                connection_info: rw.connection_info.into()
            }
        })
        .collect();

    let info = daemon_interface::WorkspaceInfo {
        name: data.name,
        path: data.local_path,
        nr_of_remote_workspaces: remote_workspaces.len(),
        remote_workspaces
    };

    WorkspaceInfoResponse { info }
}

pub(crate) fn to_list_workspaces_response(data: Vec<WorkspaceInformation>) -> ListWorkspacesResponse {
    let workspaces_overview: Vec<daemon_interface::WorkspaceOverview> = data
        .into_iter()
        .map(|workspace| {
            daemon_interface::WorkspaceOverview {
                name: workspace.name,
                path: workspace.local_path,
                nr_of_remote_workspaces: workspace.remote_workspaces.len()
            }
        })
        .collect();

    ListWorkspacesResponse {
        nr_of_workspaces: workspaces_overview.len(),
        entries: workspaces_overview
    }
}

pub(crate) fn to_list_workspace_info_response(data: Vec<WorkspaceInformation>) -> ListWorkspaceInfoResponse {
    let workspaces_info: Vec<daemon_interface::WorkspaceInfo> = data
        .into_iter()
        .map(|workspace| workspace.into())
        .collect();

    ListWorkspaceInfoResponse {
        nr_of_workspaces: workspaces_info.len(),
        entries: workspaces_info
    }
}
