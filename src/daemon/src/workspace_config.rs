use std::env;
use std::fs::File;
use std::path::PathBuf;
use log::trace;
use crate::types::{RemoteWorkspace, WorkspaceInformation};
use crate::types::errors::WsConfigError;
use crate::util::constants::WS_CONFIG_ENV_VAR;
use crate::util::error_exit;

/// Not thread-safe and no guarantees as to when changes are persisted
pub(crate) struct WorkspaceConfiguration {
    path: PathBuf
}

impl WorkspaceConfiguration {

    pub fn init() -> Self {
        let ws_config_file_path = match env::var(WS_CONFIG_ENV_VAR) {
            Ok(val) => val,
            Err(_) => {
                error_exit(Some(format!("Cannot find the workspace config file because {WS_CONFIG_ENV_VAR} is not set!")));
            }
        };

        let path: PathBuf = PathBuf::from(&ws_config_file_path);

        if !path.exists() {
            error_exit(Some(format!("There exists no workspace config file at path '{:?}'", &ws_config_file_path)));
        }

        if !path.is_file() {
            error_exit(Some(format!("Workspace config file path '{:?}' does not point to a file", &ws_config_file_path)));
        }

        WorkspaceConfiguration { path }
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

    pub fn find_by_name(&self, workspace_id: String) -> Result<Option<WorkspaceInformation>, WsConfigError> {
        trace!("find_by_name({workspace_id})");

        let config_entries = self.parse()?;

        let mut result: Vec<WorkspaceInformation> = config_entries
            .into_iter()
            .filter(|entry| entry.name == workspace_id)
            .collect();

        if result.len() == 0 {
            Ok(None)
        } else if result.len() == 1 {
            Ok(result.pop())
        } else {
            let error_msg = "Illegal state: The workspace config file contains multiple workspace entries with the same name";
            error_exit(Some(error_msg.to_string()))
        }
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
