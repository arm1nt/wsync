use std::env;
use std::fs::File;
use std::path::PathBuf;
use crate::types::{RemoteWorkspace, WorkspaceInformation};
use crate::types::errors::WsConfigError;
use crate::util::constants::WS_CONFIG_ENV_VAR;
use crate::util::error_exit;

/// Not thread-safe and no guarantees as to when changes are persisted
pub(crate) struct WorkspaceConfiguration {
    path: PathBuf
}

pub(crate) struct Error {
    msg: String
}

impl WorkspaceConfiguration {

    pub fn init() -> Self {
        let ws_config_file_path = match env::var(WS_CONFIG_ENV_VAR) {
            Ok(val) => val,
            Err(_) => {
                error_exit(Some(format!("Cannot find the workspace config file because {WS_CONFIG_ENV_VAR} is not set!")));
            }
        };

        WorkspaceConfiguration { path: PathBuf::from(ws_config_file_path) }
    }

    pub fn parse(&self) -> Result<Vec<WorkspaceInformation>, WsConfigError> {
        let file = File::open(&self.path).map_err(|e| {
            WsConfigError::new(e.to_string())
        })?;

        let ws_entries: Vec<WorkspaceInformation> = serde_json::from_reader(file).map_err(|e| {
            WsConfigError::new(e.to_string())
        })?;

        Ok(ws_entries)
    }

    pub fn add_workspace(&self, workspace: &WorkspaceInformation) -> Result<(), WsConfigError> {
        todo!()
    }

    pub fn remove_workspace(&self, workspace_id: String) -> Result<(), WsConfigError> {
        todo!()
    }

    pub fn attach_remote_workspace(&self, workspace_id: String, remote_workspace: RemoteWorkspace) -> Result<(), WsConfigError> {
        todo!()
    }

    pub fn detach_remote_workspace(&self, workspace_id: String, remote_workspace_id: String) -> Result<(), WsConfigError> {
        todo!()
    }

    fn update_config_file(&self, config_entries: Vec<WorkspaceInformation>) -> Result<(), WsConfigError> {
        todo!()
    }
}
