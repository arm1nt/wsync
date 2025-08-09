use std::collections::HashMap;
use std::process::Child;
use crate::types::errors::MonitorManagerError;
use crate::types::WorkspaceInformation;

pub(crate) struct MonitorManager {
    map: HashMap<String, Monitor>
}

struct Monitor {
    child: Child
}


impl MonitorManager {

    pub fn init() -> Self {
        MonitorManager { map: HashMap::new() }
    }

    pub fn start_monitor(&self, workspace: &WorkspaceInformation) -> Result<(), MonitorManagerError> {
        todo!()
    }

    pub fn restart_monitor(&self, workspace: &WorkspaceInformation) -> Result<(), MonitorManagerError> {
        todo!()
    }

    pub fn terminate_monitor(&self, workspace: &WorkspaceInformation) -> Result<(), MonitorManagerError> {
        todo!()
    }
}
