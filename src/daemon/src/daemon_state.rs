use std::sync::{Arc, Mutex};
use log::{info, warn};
use crate::domain::models::WorkspaceInformation;
use crate::monitor_manager::MonitorManager;
use crate::util::error_exit;
use crate::workspace_config::WorkspaceConfiguration;

pub(crate) struct DaemonState {
    pub monitor_manager: MonitorManager,
    pub ws_config: WorkspaceConfiguration
}

impl DaemonState {

    pub fn init() -> Arc<Mutex<Self>> {
        let monitor_manager = MonitorManager::init().unwrap_or_else(|e| {
            error_exit(Some(e.msg))
        });

        let ws_config = WorkspaceConfiguration::init().unwrap_or_else(|e| {
            error_exit(Some(format!("{e}")))
        });

        Arc::new(
            Mutex::new(
                DaemonState { ws_config, monitor_manager }
            )
        )
    }

    /// Restore the daemon's state by starting all workspaces specified in the ws config file
    pub fn restore(&mut self) {
        info!("Restoring daemon state from workspaces config file...");

        let mut successful_starts: usize = 0;
        let configured_workspaces: Vec<WorkspaceInformation> = self.ws_config.all();

        for workspace in configured_workspaces.iter() {
            match self.monitor_manager.start_monitor(workspace) {
                Ok(()) => {
                    successful_starts += 1;
                    info!("Successfully started monitor for '{}'", workspace.name);
                },
                Err(e) => {
                    warn!("Failed to start monitor for '{}': {e}", workspace.name);
                }
            }
        }

        info!("{}/{} monitors were started successfully!", successful_starts, configured_workspaces.len());
    }
}
