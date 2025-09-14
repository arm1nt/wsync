use std::env;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use crate::domain::models::{RemoteWorkspace, WorkspaceInformation};
use crate::util::constants::WS_CONFIG_ENV_VAR;

type Result<T> = std::result::Result<T, Error>;

pub(crate) enum Error {
    Io(String),
    Message(String)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(msg) => write!(f, "[I/O_ERROR] {msg}"),
            Error::Message(msg) => write!(f, "{msg}")
        }
    }
}

/// Not thread-safe and no guarantees as to when changes are persisted
pub(crate) struct WorkspaceConfiguration {
    pub(self) path: PathBuf,
    pub(self) cached_entries: Vec<WorkspaceInformation>
}

impl WorkspaceConfiguration {

    pub(crate) fn init() -> Result<Self> {

        let ws_config_file_path = env::var(WS_CONFIG_ENV_VAR).map_err(|_| {
            Error::Message(format!("{WS_CONFIG_ENV_VAR} is not set!"))
        })?;

        let path = PathBuf::from(&ws_config_file_path);
        validate_config_file_path(&path)?;

        let config_entries = Self::read_file(&path)?;
        Ok( Self { path, cached_entries: config_entries })
    }

    pub(crate) fn all(&self) -> Vec<WorkspaceInformation> {
        self.cached_entries.clone()
    }

    pub(crate) fn find_by_name(&self, workspace_id: &String) -> Option<WorkspaceInformation> {
        self.cached_entries
            .iter()
            .find(|entry| entry.name == *workspace_id)
            .cloned()
    }

    fn find_by_name_mut(&mut self, workspace_id: &String) -> Option<&mut WorkspaceInformation> {
        self.cached_entries.iter_mut().find(|entry| entry.name == *workspace_id)
    }

    pub(crate) fn add_workspace(&mut self, workspace: WorkspaceInformation) -> Result<()> {
        let conflicting_entries: Vec<&WorkspaceInformation> = self.cached_entries
            .iter()
            .filter(|entry| entry.name == workspace.name || entry.local_path == workspace.local_path)
            .collect();

        if !conflicting_entries.is_empty() {
            return Err(Error::Message(
                format!(
                    "Local workspace with this name or at this local path already exists: {:?}",
                    conflicting_entries
                )
            ));
        }

        self.cached_entries.push(workspace);
        self.write_file()?;
        Ok(())
    }

    pub(crate) fn remove_workspace(&mut self, workspace_id: String) -> Result<()> {
        let elements_before = self.cached_entries.len();
        self.cached_entries.retain(|entry| entry.name != workspace_id);

        if self.cached_entries.len() == elements_before {
            return Err(Error::Message(format!("No workspace named '{workspace_id}' found")));
        }

        self.write_file()?;
        Ok(())
    }

    pub(crate) fn attach_remote_workspace(
        &mut self,
        workspace_id: String,
        remote_workspace: RemoteWorkspace
    ) -> Result<()> {
        let entry = self.find_by_name_mut(&workspace_id).ok_or(
            Error::Message(format!("No local workspace named '{}' exists", workspace_id))
        )?;

        if entry.remote_workspaces.iter().any(|rw| rw.name == remote_workspace.name) {
            return Err(Error::Message(
                format!(
                    "A remote workspace named '{}' is already attached to the local workspace '{}'",
                    remote_workspace.name,
                    workspace_id
                )
            ));
        }

        entry.remote_workspaces.push(remote_workspace);
        self.write_file()?;
        Ok(())
    }

    pub(crate) fn detach_remote_workspace(
        &mut self,
        workspace_id: String,
        remote_workspace_id: String
    ) -> Result<()> {
        let entry = self.find_by_name_mut(&workspace_id).ok_or(
            Error::Message(format!("No local workspace named '{}' exists", workspace_id))
        )?;

        let rw_before = entry.remote_workspaces.len();
        entry.remote_workspaces.retain(|rw| rw.name != remote_workspace_id);

        if entry.remote_workspaces.len() == rw_before {
            return Err(Error::Message(
                format!(
                    "No remote workspace named '{}' is attached to the local workspace '{}'",
                    remote_workspace_id,
                    workspace_id
                )
            ));
        }

        self.write_file()?;
        Ok(())
    }

    fn read_file(path: &Path) -> Result<Vec<WorkspaceInformation>> {
        let file = File::open(path).map_err(|e|
            Error::Io(format!("Opening ws config file failed: {e}"))
        )?;

        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e|
            Error::Io(format!("Parsing ws config file failed: {e}"))
        )
    }

    fn write_file(&self) -> Result<()> {
        // Todo: Before writing out changes, write to a temp file

        let file = File::options()
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|e| {
                Error::Io(format!("Unable to update workspaces config file: {e:?}"))
            })?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.cached_entries).map_err(|e| {
            Error::Io(format!("Unable to update ws config file: {e}"))
        })?;

        writer.flush().map_err(|e|
            Error::Io(format!("Unable to write ws config changes to file: {e}"))
        )?;

        Ok(())
    }

}

fn validate_config_file_path(path: &PathBuf) -> Result<()> {

    if !path.exists() {
        return Err(Error::Message(format!("'{:?}' does not exist", path)));
    }

    if !path.is_file() {
        return Err(Error::Message(format!("'{:?}' does not refer to a file", path)));
    }

    Ok(())
}
