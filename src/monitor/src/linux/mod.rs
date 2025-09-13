use crate::linux::inotify::{init_inotify_instance, listen_for_events};
use crate::linux::monitor_state::MonitorState;
use crate::linux::signals::install_signal_handlers;
use crate::models::WorkspaceInfo;
use crate::util::error_exit;

pub(self) mod monitor_state;
pub(self) mod inotify;
pub(self) mod signals;

pub(crate) fn run_fs_listener(workspace_info: WorkspaceInfo) {

    let _ = install_signal_handlers().unwrap_or_else(|e| {
        error_exit(Some(format!("Unable to install signal handler(s): {}", e.msg)))
    });

    let mut state = MonitorState::default(&workspace_info);

    let mut inotify = init_inotify_instance(&workspace_info, &mut state).unwrap_or_else(|e| {
        error_exit(Some(format!("Unable to initialize inotify instance: {}", e.msg)))
    });

    listen_for_events(&mut inotify, &mut state);

    let _ = inotify.close().unwrap_or_else(|e| {
        error_exit(Some(format!("Failed to close inotify instance fd: {e}")))
    });
}
