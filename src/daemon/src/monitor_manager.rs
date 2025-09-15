use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use log::debug;
use crate::domain::models::WorkspaceInformation;
use crate::util::constants::MONITOR_EXECUTABLE_ENV_VAR;

type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Error {
    pub(crate) msg: String
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error {
    pub(crate) fn new(msg: String) -> Self {
        Error { msg }
    }
}

pub(crate) struct MonitorManager {
    // Only to be directly accessed by the watchdog
    pub(crate) ws_id_to_monitor: HashMap<String, Child>,
    pub(self) monitor_executable: String
}

impl MonitorManager {

    pub(crate) fn init() -> Result<Self> {
        let monitor_executable = env::var(MONITOR_EXECUTABLE_ENV_VAR).map_err(|_| {
            Error::new(format!("{MONITOR_EXECUTABLE_ENV_VAR} env var not set"))
        })?;

        let executable_path = Path::new(&monitor_executable);
        if !executable_path.exists() {
            return Err(Error::new(
                format!("Monitor executable not found at '{monitor_executable}'")
            ));
        }

        Ok( MonitorManager { ws_id_to_monitor: HashMap::new(), monitor_executable } )
    }

    pub(crate) fn start_monitor(&mut self, workspace: &WorkspaceInformation) -> Result<()> {

        if workspace.remote_workspaces.is_empty() {
            // Don't spawn monitor as there are no remote ws to sync with
            return Ok(())
        }

        match self.ws_id_to_monitor.entry(workspace.name.clone()) {
            Entry::Occupied(_) => {
                Err(Error::new(format!(
                    "A monitor for workspace '{}' already exists!", workspace.name
                )))
            },
            Entry::Vacant(entry) => {

                let serialized_ws = serde_json::to_string(workspace).map_err(|e| {
                    Error::new(format!(
                        "Unable to spawn monitor for '{}' because serializing the workspace information failed: {e}",
                        workspace.name
                    ))
                })?;

                let mut child = Command::new(&self.monitor_executable)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .map_err(|e| {
                        Error::new(format!(
                            "Spawning monitor for '{}' failed: {e}", workspace.name
                        ))
                    })?;

                match child.stdin.take() {
                    Some(mut stdin) => {
                        let res = stdin.write_all(serialized_ws.as_bytes()).map_err(|e| {
                            Error::new(format!(
                                "Unable to pass serialized workspace information to spawned monitor: {e}"
                            ))
                        });

                        if let Err(e) = res {
                            let _ = Self::kill_monitor(child);
                            return Err(e);
                        }
                    },
                    None => {
                        let _ = Self::kill_monitor(child);
                        return Err(Error::new(
                            "Failed to open stdin of spawned monitor to pass it the workspace information".to_string()
                        ));
                    }
                }

                entry.insert(child);
                Ok(())
            }
        }
    }

    pub(crate) fn restart_monitor(&mut self, workspace: &WorkspaceInformation) -> Result<()> {

        if !self.ws_id_to_monitor.contains_key(&workspace.name) {
            return self.start_monitor(workspace);
        }

        self.terminate_monitor(&workspace.name)?;
        self.start_monitor(workspace)?;

        Ok(())
    }

    pub(crate) fn terminate_monitor(&mut self, workspace_id: &String) -> Result<()> {

        let monitor = match self.ws_id_to_monitor.remove(workspace_id) {
            Some(monitor) => monitor,
            None => {
                debug!("Nothing to terminate because no monitor is running for '{}'", workspace_id);
                return Ok(());
            }
        };

        Self::kill_monitor(monitor)?;

        Ok(())
    }

    fn kill_monitor(mut child: Child) -> Result<()> {

        // Monitor processes don't hold any state that needs to be persisted or synced, so we don't
        // need to let them shut down gracefully.
        child.kill().map_err(|e| {
            Error::new(format!("Unable to kill monitor process: {e}"))
        })?;

        let _ = child.wait();

        Ok(())
    }
}
