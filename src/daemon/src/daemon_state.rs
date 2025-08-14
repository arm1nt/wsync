use std::sync::{Arc, Mutex};
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
    pub fn restore(&self) {
        //todo!()
    }
}
