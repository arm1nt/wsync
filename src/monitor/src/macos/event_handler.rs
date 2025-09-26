use std::{path::PathBuf, sync::mpsc::Receiver};
use log::{debug, error, info, warn};
use notify::Event;
use crate::{models::{Error, WorkspaceInfo}, sync::synchronize_workspace, util::fs::strip_ws_root_prefix};

type Result<T> = std::result::Result<T, Error>;

fn handle_event(event: Event, ws_info: &WorkspaceInfo) -> Result<()> {
    debug!("{:?}", event);

    for target_path in event.paths {

        let mut rel_path_to_sync: Option<PathBuf> = None;

        if target_path == ws_info.local_path {
            if event.kind.is_remove() {
                return Err(Error::new("Workspace was removed"));
            }
        } else {
            // Get the parent directory path, because the directory containing the resource in question should be synced.
            let parent_path = match target_path.parent() {
                Some(parent_path) => parent_path.to_path_buf(),
                None => continue
            };

            rel_path_to_sync = match strip_ws_root_prefix(&ws_info.local_path, &parent_path) {
                Ok(stripped) => stripped,
                Err(e) => {
                    warn!("Unable to strip ws root form '{:?}': {}", parent_path, e);
                    continue;
                }
            };
        }

        synchronize_workspace(ws_info, rel_path_to_sync.as_ref()).map_err(|e| {
            Error::new(format!("{e:?}"))
        })?;
    }

    Ok(())
}

pub(in crate::macos) fn listen_for_events(rx: Receiver<notify::Result<Event>>, ws_info: &WorkspaceInfo) {

    for result in rx {
        match result {
            Ok(event) => {
                if let Err(error) = handle_event(event, ws_info) {
                    error!("Error handling fs event: {error}");
                    break;
                }
            },
            Err(e) => {
                error!("An error occurred while reading fs events: {:?}", e);
                error!("Terminating fs-event-reader loop...");
                break;
            }
        }
    }

}
