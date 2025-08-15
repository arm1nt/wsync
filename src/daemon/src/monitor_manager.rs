use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::collections::hash_map::Entry;
use crate::domain::errors::MonitorManagerError;
use crate::domain::models::WorkspaceInformation;
use crate::util::constants::MONITOR_EXECUTABLE_ENV_VAR;

pub(crate) struct MonitorManager {
    // Only to be directly accessed by the watchdog
    pub(crate) map: HashMap<String, Child>,
    monitor_executable: String
}

// Todo: Implement a watchdog that checks if a monitor crashed, and if so, performs
//  cleanup tasks, etc.

impl MonitorManager {

    pub fn init() -> Result<Self, MonitorManagerError> {
        let monitor_executable = env::var(MONITOR_EXECUTABLE_ENV_VAR).map_err(|_| {
            MonitorManagerError::new(format!("{MONITOR_EXECUTABLE_ENV_VAR} env var not set"))
        })?;

        let executable_path = Path::new(&monitor_executable);
        if !executable_path.exists() {
            return Err(MonitorManagerError::new(format!(
                "Monitor executable not found at '{monitor_executable}'"
            )));
        }

        Ok( MonitorManager { map: HashMap::new(), monitor_executable } )
    }

    pub fn start_monitor(&mut self, workspace: &WorkspaceInformation) -> Result<(), MonitorManagerError> {

        if workspace.remote_workspaces.is_empty() {
            // Don't spawn monitor as there are no remote ws to sync with
            return Ok(())
        }

        match self.map.entry(workspace.name.clone()) {
            Entry::Occupied(_) => {
                Err(MonitorManagerError::new(format!(
                    "A monitor for workspace '{}' already exists!", workspace.name
                )))
            },
            Entry::Vacant(entry) => {

                let serialized_ws = serde_json::to_string(workspace).map_err(|e| {
                    MonitorManagerError::new(format!(
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
                        MonitorManagerError::new(format!(
                            "Spawning monitor for '{}' failed: {e}", workspace.name
                        ))
                    })?;

                match child.stdin.take() {
                    Some(mut stdin) => {
                        stdin.write_all(serialized_ws.as_bytes()).map_err(|e| {
                            MonitorManagerError::new(format!(
                                "Unable to pass serialized workspace information to spawned monitor: {e}"
                            ))
                        })?;
                    },
                    None => {
                        let _ = Self::kill_monitor(child);
                        return Err(MonitorManagerError::new(
                            "Failed to open stdin of spawned monitor to pass it the workspace information".to_string()
                        ));
                    }
                }

                entry.insert(child);
                Ok(())
            }
        }
    }

    pub fn restart_monitor(&mut self, workspace: &WorkspaceInformation) -> Result<(), MonitorManagerError> {

        if !self.map.contains_key(&workspace.name) {
            return self.start_monitor(workspace);
        }

        self.terminate_monitor(&workspace.name)?;
        self.start_monitor(workspace)?;

        Ok(())
    }

    pub fn terminate_monitor(&mut self, workspace_id: &String) -> Result<(), MonitorManagerError> {

        let monitor = match self.map.remove(workspace_id) {
            Some(monitor) => monitor,
            None => {
                return Err(MonitorManagerError::new(format!(
                    "Cannot terminate monitor of '{workspace_id}' because no monitor exists"
                )));
            }
        };

        Self::kill_monitor(monitor)?;

        Ok(())
    }

    fn kill_monitor(mut child: Child) -> Result<(), MonitorManagerError> {

        // Monitor processes don't hold any state, so we don't need to let them shut down gracefully
        child.kill().map_err(|e| {
            MonitorManagerError::new(format!("Unable to kill monitor process: {e}"))
        })?;

        let _ = child.wait();

        Ok(())
    }
}
