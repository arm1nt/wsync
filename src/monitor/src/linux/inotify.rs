use std::cmp::Ordering;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::PathBuf;
use inotify::{Event, EventMask, Inotify, WatchDescriptor, WatchMask};
use log::{debug, error, trace, warn};
use crate::linux::monitor_state;
use crate::linux::monitor_state::{MonitorState, WatchMetadata};
use crate::models::{Error, WorkspaceInfo};
use crate::util::fs::{concat_paths, get_subdir_names};

type Result<T> = std::result::Result<T, Error>;

fn get_watch_mask() -> WatchMask {
    WatchMask::MODIFY |
    WatchMask::CLOSE_WRITE |
    WatchMask::CREATE |
    WatchMask::DELETE |
    WatchMask::DELETE_SELF |
    WatchMask::MOVE |
    WatchMask::MOVE_SELF |
    WatchMask::EXCL_UNLINK |
    // Since we only add watches for directories, we will not be notified if a file contained in the
    // ws tree is modified via a symlink outside the workspace. But this is outside wsync's scope anyway.
    WatchMask::ONLYDIR
}

fn get_parent_pathbuf(path: &PathBuf) -> Result<PathBuf> {
    let parent_path = path
        .parent()
        .ok_or(Error::new(format!("Unable to get parent of '{:?}'", path)))?;

    Ok(parent_path.to_path_buf())
}

fn get_event_name_pathbuf(event: &Event<&OsStr>) -> Result<Option<PathBuf>> {
    if event.name.is_none() {
        return Ok(None);
    }

    let stringified_os_str = match event.name.unwrap().to_str() {
        Some(val) => val,
        None => {
            return Err(Error::new(
                format!("Unable to stringify OS str '{:?}'", event.name.unwrap())
            ));
        }
    };

    let path = PathBuf::from(stringified_os_str);
    Ok(Some(path))
}

fn remove_watches_recursively(
    inotify: &mut Inotify,
    state: &mut MonitorState,
    watch_descriptor: &WatchDescriptor
) -> Result<()> {

    let watch_metadata = state.get_metadata(watch_descriptor)
        .ok_or(Error::new(format!("No metadata stored for wd '{:?}'", watch_descriptor)))?
        .clone();

    let _ = inotify.watches().remove(watch_descriptor.clone());
    debug!("Called 'inotify_rm_watch' for wd '{:?}'", watch_descriptor);

    // Remove the wd entry from the parent directory's list of child watches
    if watch_metadata.relative_path.is_some() {
        let parent_path = get_parent_pathbuf(&watch_metadata.path)?;
        let mut parent_md = state
            .get_metadata_mut_from_path(&parent_path)
            .ok_or(Error::new(format!("No metadata stored for '{:?}'", parent_path)))?;

        parent_md.child_watches.retain(|e| e != watch_descriptor);
    }

    // Remove the watches of all descendant directories
    for child_wd in watch_metadata.child_watches.iter() {
        remove_watches_recursively(inotify, state, child_wd)?
    }

    state.rm_watch_metadata(watch_descriptor).map_err(|e| {
        Error::new(format!("Unable to remove watch metadata: {e}"))
    })?;

    Ok(())
}

fn add_watch(
    inotify: &mut Inotify,
    state: &mut MonitorState,
    ws_root_path: &PathBuf,
    relative_path: Option<&PathBuf>
) -> Result<()> {
    let full_path = concat_paths(Some(ws_root_path), relative_path)?;

    let watch_descriptor = inotify
        .watches()
        .add(&full_path, get_watch_mask())
        .map_err(|e| {
            Error::new(format!("Failed to register watch for '{:?}': {e}", full_path))
        })?;

    let metadata = WatchMetadata::initialize(
        &watch_descriptor,
        ws_root_path,
        relative_path,
        &full_path
    );
    state.add_watch_metadata(&full_path, &watch_descriptor, &metadata);

    // If applicable, add the wd to the parent directory's list of watch descriptors. This way, we
    // can remove the watches of all descendant dirs when e.g. a dir is moved out of the workspace.
    if ws_root_path.cmp(&full_path) == Ordering::Equal {
        debug!("Added watch for '{:?}'!", full_path);
        return Ok(());
    }

    let parent_path = get_parent_pathbuf(&full_path)?;
    let parent_md = state
        .get_metadata_mut_from_path(&parent_path)
        .ok_or(Error::new(format!("No metadata stored for '{:?}'", parent_path)))?;

    parent_md.child_watches.push(watch_descriptor.clone());

    debug!("Added watch for '{:?}'!", full_path);
    Ok(())
}

fn add_watches_recursively(
    inotify: &mut Inotify,
    state: &mut MonitorState,
    ws_root_path: PathBuf,
    relative_path: Option<PathBuf>
) -> Result<()> {
    let mut relative_subdir_paths: Vec<PathBuf> = vec![];

    if let Some(relative_path) = relative_path {
        relative_subdir_paths.push(relative_path);
    }

    loop {
        let relative_subdir_path = relative_subdir_paths.pop();
        let full_path = concat_paths(Some(&ws_root_path), relative_subdir_path.as_ref())?;

        add_watch(inotify, state, &ws_root_path, relative_subdir_path.as_ref())?;

        for subdir_name in get_subdir_names(&full_path)? {
            let path = concat_paths(relative_subdir_path.as_ref(), Some(&subdir_name))?;
            relative_subdir_paths.push(path);
        }

        if relative_subdir_paths.is_empty() {
            break;
        }
    }

    debug!("Done adding watches!");
    Ok(())
}

pub(super) fn init_inotify_instance(ws_info: &WorkspaceInfo, state: &mut MonitorState) -> Result<Inotify> {
    let mut inotify = Inotify::init().map_err(|e| {
        Error::new(format!("Failed to create inotify instance: {e}"))
    })?;

    let ws_root_path = ws_info.local_path.clone();
    add_watches_recursively(&mut inotify, state, ws_root_path, None)?;

    Ok(inotify)
}

fn handle_inotify_event(event: Event<&OsStr>, inotify: &mut Inotify, state: &mut MonitorState) -> Result<()> {
    debug!("{:?}", event);

    match state.contains_wd(&event.wd) {
        Ok(contained) if !contained => {
            // Chances are that this is an old event that is still enqueued, but the associated
            // watch was already removed.
            debug!("No metadata stored for watch descriptor '{:?}'. Watch was already removed!", event.wd);
            return Ok(())
        },
        Err(e) => {
            return Err(Error::new(
                format!("Error while checking if the monitor state contains information about wd '{:?}': {e}", event.wd)
            ));
        },
        _ => {}
    }

    // IN_OVERFLOW  --> maybe check if our dir-tree is still up-to-date?
    // IN_UNMOUNT   --> maybe terminate? ignore?

    let metadata = state.get_metadata(&event.wd).unwrap().clone();
    debug!("{metadata}");

    if event.mask.contains(EventMask::IGNORED) {
        debug!(
            "Received '{:?}' for a watch that was implicitly removed. Removing associated metadata!",
            EventMask::IGNORED
        );

        if let Err(error @ monitor_state::Error::InconsistentState(_)) = state.rm_watch_metadata(&event.wd) {
            return Err(Error::new(
                format!("Error while deleting metadata of implicitly removed watch: {error}")
            ));
        }
    }

    let affected_dir_path = concat_paths(Some(&metadata.ws_root_path), metadata.relative_path.as_ref())?;

    let event_name_path = get_event_name_pathbuf(&event)?;
    let resource_path = concat_paths(Some(&affected_dir_path), event_name_path.as_ref())?;
    // Needs to be handled separately, as e.g. for an 'IN_DELETE_SELF' event on the ws root, the
    // relative path is none and there is also no event name value present.
    let relative_resource_path = if metadata.relative_path.is_none() && event_name_path.is_none() {
        None
    } else {
        Some(concat_paths(metadata.relative_path.as_ref(), event_name_path.as_ref())?)
    };

    if event.mask.contains(EventMask::DELETE_SELF) || event.mask.contains(EventMask::MOVE_SELF) {

        if metadata.relative_path.is_some() {
            // These two events are only relevant for the workspace root, as they require extra
            // handling + the ws root has no parent. For all ws subdirectories, we handle the
            // non-self event also emitted for the parent directory.
            return Ok(());
        }

        // Todo:
    }

    if event.mask.contains(EventMask::CREATE) || event.mask.contains(EventMask::MOVED_TO) {

        if !event.mask.contains(EventMask::ISDIR) {
            // File creation causes both an 'IN_CREATE' and an 'IN_CLOSE_WRITE' event to be emitted.
            // To prevent unnecessary duplicate syncs with the remote system(s), we ignore create
            // events for files.
            return Ok(());
        }

        debug!("Registering watches for subdirectory (tree) '{:?}'", relative_resource_path);
        add_watches_recursively(inotify, state, metadata.ws_root_path.clone(), relative_resource_path.clone())?;
    }

    if event.mask.contains(EventMask::ISDIR)
        && (event.mask.contains(EventMask::DELETE) || event.mask.contains(EventMask::MOVED_FROM)) {

        let dir_wd = state
            .get_wd(&resource_path)
            .ok_or(Error::new(format!("No metadata stored for '{:?}'", resource_path)))?
            .clone();
        let _ = remove_watches_recursively(inotify, state, &dir_wd);
    }

    // Todo: do sync
    warn!("Should sync '{:?}' here", metadata.relative_path.as_ref());
    Ok(())
}

pub(super) fn listen_for_events(inotify: &mut Inotify, state: &mut MonitorState) {
    let mut buffer = [0; 4096];

    'event_reader: loop {

        let events = match inotify.read_events_blocking(&mut buffer) {
            Ok(events) => events,
            Err(error) if error.kind() == ErrorKind::Interrupted => {
                warn!("Monitor loop reading inotify events was interrupted: {error}");
                warn!("Terminating inotify-event reader loop...");
                break;
            },
            Err(error) => {
                error!("An error occurred while reading fs events: {error}");
                error!("Terminating inotify-event reader loop...");
                break;
            }
        };

        for event in events {
            if let Err(error) = handle_inotify_event(event, inotify, state) {
                error!("Error handling inotify event: {}", error.msg);
                break 'event_reader;
            }
        }
    }
}
