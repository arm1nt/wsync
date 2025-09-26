use std::sync::mpsc;
use notify::{Event, Result, Watcher};
use crate::{macos::event_handler::listen_for_events, models::WorkspaceInfo, util::error_exit};

pub(self) mod event_handler;

pub(crate) fn run_fs_listener(workspace_info: WorkspaceInfo) {

    let (tx, rx) = mpsc::channel::<Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx).unwrap_or_else(|e| {
        error_exit(Some(format!("Unable to create watcher: {:?}", e)))
    });

    watcher.watch(&workspace_info.local_path, notify::RecursiveMode::Recursive).unwrap_or_else(|e| {
        error_exit(Some(format!("Unable to start watcher: {:?}", e)))
    });

    listen_for_events(rx, &workspace_info);
}
