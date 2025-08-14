use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use crate::domain::models::{RemoteWorkspace, WorkspaceInformation};
use crate::domain::errors::WsConfigError;
use crate::util::constants::WS_CONFIG_ENV_VAR;

/// Not thread-safe and no guarantees as to when changes are persisted
pub(crate) struct WorkspaceConfiguration {
    path: PathBuf,
    cached_entries: Vec<WorkspaceInformation>
}

impl WorkspaceConfiguration {

    pub fn init() -> Result<Self, WsConfigError> {
        let ws_config_file_path = env::var(WS_CONFIG_ENV_VAR).map_err(|e| {
            WsConfigError::Message(format!("{WS_CONFIG_ENV_VAR} is not set!"))
        })?;

        let path = PathBuf::from(&ws_config_file_path);
        if !path.exists() {
            return Err(WsConfigError::Message(
                format!("Workspace config file does not exist at path '{ws_config_file_path:?}'")
            ));
        }
        if !path.is_file() {
            return Err(WsConfigError::Message(
                format!("Workspace config path is not a file '{ws_config_file_path:?}'")
            ));
        }

        let config_entries: Vec<WorkspaceInformation> = Self::read_file(&path)?;

        Ok( Self { path, cached_entries: config_entries })
    }

    pub fn all(&self) -> Vec<WorkspaceInformation> {
        self.cached_entries.clone()
    }

    pub fn find_by_name(&self, workspace_id: &String) -> Option<WorkspaceInformation> {
        self.cached_entries
            .iter()
            .find(|entry| entry.name == *workspace_id)
            .map(|e| e.clone())
    }

    fn find_by_name_mut(&mut self, workspace_id: String) -> Option<&mut WorkspaceInformation> {
        self.cached_entries.iter_mut().find(|entry| entry.name == workspace_id)
    }

    pub fn add_workspace(&mut self, workspace: WorkspaceInformation) -> Result<(), WsConfigError> {
        let conflicting_entries: Vec<&WorkspaceInformation> = self.cached_entries
            .iter()
            .filter(|entry| entry.name == workspace.name || entry.local_path == workspace.local_path)
            .collect();

        if !conflicting_entries.is_empty() {
            return Err(WsConfigError::Message(
                format!("Local workspace with this name or at this local path already exists: {:?}", conflicting_entries)
            ));
        }

        self.cached_entries.push(workspace);
        self.write_file()?;
        Ok(())
    }

    pub fn remove_workspace(&mut self, workspace_id: String) -> Result<(), WsConfigError> {
        let elements_before = self.cached_entries.len();
        self.cached_entries.retain(|entry| entry.name != workspace_id);

        if self.cached_entries.len() == elements_before {
            return Err(WsConfigError::Message(
                format!("No workspace named '{workspace_id}' found")
            ));
        }

        self.write_file()?;
        Ok(())
    }

    pub fn attach_remote_workspace(
        &mut self,
        workspace_id: String,
        remote_workspace: RemoteWorkspace
    ) -> Result<(), WsConfigError> {
        let entry = self.find_by_name_mut(workspace_id.clone()).ok_or_else(|| {
            WsConfigError::Message(format!("No local workspace named '{workspace_id}' exists"))
        })?;

        if entry.remote_workspaces.iter().any(|rw| rw.name == remote_workspace.name) {
            return Err(WsConfigError::Message(
                format!("Remote Workspace named '{}' is already attached to local workspace '{workspace_id}", remote_workspace.name)
            ));
        }

        entry.remote_workspaces.push(remote_workspace);
        self.write_file()?;
        Ok(())
    }

    pub fn detach_remote_workspace(
        &mut self,
        workspace_id: String,
        remote_workspace_id: String
    ) -> Result<(), WsConfigError> {
        let entry = self.find_by_name_mut(workspace_id.clone()).ok_or_else(|| {
            WsConfigError::Message(format!("No local workspace named '{workspace_id}' exists"))
        })?;

        let rw_before = entry.remote_workspaces.len();
        entry.remote_workspaces.retain(|rw| rw.name != remote_workspace_id);

        if entry.remote_workspaces.len() == rw_before {
            return Err(WsConfigError::Message(
                format!("No remote workspace named '{remote_workspace_id}' attached to local workspace '{workspace_id}'")
            ));
        }

        self.write_file()?;
        Ok(())
    }

    fn read_file(path: &Path) -> Result<Vec<WorkspaceInformation>, WsConfigError> {
        let file = File::open(path).map_err(|e|
            WsConfigError::Io(format!("Opening ws config file failed: {e}"))
        )?;

        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e|
            WsConfigError::Io(format!("Parsing ws config file failed: {e}"))
        )
    }

    fn write_file(&self) -> Result<(), WsConfigError> {
        // Todo: Before writing out changes, write to a temp file

        let file = File::options()
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|e| {
                WsConfigError::Io(format!("Unable to update workspaces config file: {e:?}"))
            })?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.cached_entries).map_err(|e| {
            WsConfigError::Io(format!("Unable to update ws config file: {e}"))
        })?;

        writer.flush().map_err(|e|
            WsConfigError::Io(format!("Unable to write ws config changes to file: {e}"))
        )?;

        Ok(())
    }

}
