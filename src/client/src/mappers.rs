use std::fmt::{Debug, Display, Formatter};
use serde::Serialize;
use serde_json::Value;
use crate::cli::{
    WorkspaceInfoArgs,
    AddWorkspaceArgs,
    RemoveWorkspaceArgs,
    AttachRemoteWorkspaceSubcommands,
    SshArgs,
    RsyncArgs,
    DetachRemoteWorkspaceArgs,
    Cli,
    Command,
    HostInfo,
};
use daemon_interface::{request, ConnectionInfo};
use daemon_interface::request::{
    AddWorkspaceRequest,
    AttachRemoteWorkspaceRequest,
    CommandRequest,
    DetachRemoteWorkspaceRequest,
    RemoveWorkspaceRequest,
    WorkspaceInfoRequest,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) struct ClientRequest {
    pub(crate) command_request: Value,
    pub(crate) command_data: Option<Value>
}

#[derive(Debug)]
pub(crate) struct Error {
    pub(self) msg: String
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ClientRequest {

    pub(crate) fn get_client_request(cli: Cli) -> Result<Self> {
        match cli.command {
            Command::WorkspaceInfo(args) => {
                Ok(Self::get_workspace_info_request(args)?)
            }
            Command::ListWorkspaces(_) => {
                Ok(Self::get_list_workspaces_request()?)
            }
            Command::ListWorkspaceInfo(_) => {
                Ok(Self::get_list_workspace_info_request()?)
            }
            Command::AddWorkspace(args) => {
                Ok(Self::get_add_workspace_request(args)?)
            }
            Command::RemoveWorkspace(args) => {
                Ok(Self::get_remove_workspace_request(args)?)
            }
            Command::AttachRemoteWorkspace(sub_command) => {
                match sub_command.command {
                    AttachRemoteWorkspaceSubcommands::Ssh(args) => {
                        Ok(Self::get_ssh_attach_remote_workspace_request(args)?)
                    }
                    AttachRemoteWorkspaceSubcommands::Rsync(args) => {
                        Ok(Self::get_rsync_attach_remote_workspace_request(args)?)
                    }
                }
            }
            Command::DetachRemoteWorkspace(args) => {
                Ok(Self::get_detach_remote_workspace_request(args)?)
            }
        }
    }

    fn get_workspace_info_request(args: WorkspaceInfoArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::WorkspaceInfo)?;
        let command_data = Self::workspace_info_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn workspace_info_args_to_json(args: WorkspaceInfoArgs) -> Result<Value> {
        let data = WorkspaceInfoRequest {
            name: args.name
        };

        Ok(Self::get_command_data(data)?)
    }

    fn get_list_workspaces_request() -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::ListWorkspaces)?;

        Ok(Self { command_request, command_data: None })
    }

    fn get_list_workspace_info_request() -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::ListWorkspaceInfo)?;

        Ok(Self { command_request, command_data: None })
    }

    fn get_add_workspace_request(args: AddWorkspaceArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::AddWorkspace)?;
        let command_data = Self::add_workspace_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn add_workspace_args_to_json(args: AddWorkspaceArgs) -> Result<Value> {
        let data = AddWorkspaceRequest {
            name: args.name,
            path: args.path,
        };

        Ok(Self::get_command_data(data)?)
    }

    fn get_remove_workspace_request(args: RemoveWorkspaceArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::RemoveWorkspace)?;
        let command_data = Self::remove_workspace_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn remove_workspace_args_to_json(args: RemoveWorkspaceArgs) -> Result<Value> {
        let data = RemoveWorkspaceRequest {
            name: args.name,
        };

        Ok(Self::get_command_data(data)?)
    }

    fn get_ssh_attach_remote_workspace_request(args: SshArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::AttachRemoteWorkspace)?;
        let command_data = Self::ssh_attach_remote_workspace_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn ssh_attach_remote_workspace_args_to_json(args: SshArgs) -> Result<Value> {
        let connection_info = if args.host_alias.is_some() {
            ConnectionInfo::HostAlias { host_alias: args.host_alias.unwrap() }
        } else {
            ConnectionInfo::Ssh {
                host: Self::unwrap_host_info(args.host_info),
                port: Some(args.port),
                username: args.user,
                identity_file: args.identity_file,
            }
        };

        let data = AttachRemoteWorkspaceRequest {
            local_workspace_name: args.args.workspace_name,
            remote_workspace_name: args.args.remote_workspace_name,
            remote_workspace_path: args.args.remote_path,
            connection_info,
        };

        Ok(Self::get_command_data(data)?)
    }

    fn get_rsync_attach_remote_workspace_request(args: RsyncArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::AttachRemoteWorkspace)?;
        let command_data = Self::rsync_attach_remote_workspace_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn rsync_attach_remote_workspace_args_to_json(args: RsyncArgs) -> Result<Value> {
        let connection_info = ConnectionInfo::RsyncDaemon {
            host: Self::unwrap_host_info(args.host_info),
            port: Some(args.port),
            username: args.user,
        };

        let data = AttachRemoteWorkspaceRequest {
            local_workspace_name: args.args.workspace_name,
            remote_workspace_name: args.args.remote_workspace_name,
            remote_workspace_path: args.args.remote_path,
            connection_info,
        };

        Ok(Self::get_command_data(data)?)
    }

    fn unwrap_host_info(host_info: HostInfo) -> String {
        if host_info.hostname.is_some() {
            host_info.hostname.unwrap()
        } else {
            host_info.ip_addr.unwrap().to_string()
        }
    }

    fn get_detach_remote_workspace_request(args: DetachRemoteWorkspaceArgs) -> Result<Self> {
        let command_request = Self::get_command_request(request::Command::DetachRemoteWorkspace)?;
        let command_data = Self::detach_remote_workspace_args_to_json(args)?;

        Ok(Self { command_request, command_data: Some(command_data) })
    }

    fn detach_remote_workspace_args_to_json(args: DetachRemoteWorkspaceArgs) -> Result<Value> {
        let data = DetachRemoteWorkspaceRequest {
            local_workspace_name: args.workspace_name,
            remote_workspace_name: args.remote_workspace_name,
        };

        Ok(Self::get_command_data(data)?)
    }

    fn get_command_request(command: request::Command) -> Result<Value> {
        let command_request = CommandRequest {
            command: command.to_string()
        };

        Self::get_json_value(command_request).map_err(|e| {
            Error {
                msg: format!("Failed to create command request for command '{}': {e}", command)
            }
        })
    }

    fn get_command_data<T: Serialize>(data: T) -> Result<Value> {
        Self::get_json_value(data).map_err(|e| {
            Error {
                msg: format!("Failed to serialize request data: {e}")
            }
        })
    }

    fn get_json_value<T: Serialize>(data: T) -> std::result::Result<Value, serde_json::Error> {
        Ok(serde_json::to_value(&data)?)
    }

}
