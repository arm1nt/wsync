use std::sync::{Arc, Mutex};
use crate::monitor_manager::MonitorManager;
use crate::workspace_config::WorkspaceConfiguration;

pub(crate) struct DaemonState {
    monitor_manager: MonitorManager,
    ws_config: WorkspaceConfiguration
}

impl DaemonState {

    pub fn init() -> Arc<Mutex<Self>> {
        let ws_config = WorkspaceConfiguration::init();
        let monitor_manager = MonitorManager::init();

        Arc::new(
            Mutex::new(
                DaemonState { ws_config, monitor_manager }
            )
        )
    }

    /// Restore the daemon's state by starting all workspaces specified in the ws config file
    pub fn restore(&self) {
        todo!()
    }
}
