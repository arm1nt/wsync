use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use inotify::WatchDescriptor;
use crate::models::WorkspaceInfo;

pub(super) enum Error {
    InconsistentState(String),
    NotFound(String),
    Error(String)
}

pub(super) struct MonitorState<'ws_info> {
    pub(super) workspace_info: &'ws_info WorkspaceInfo,
    pub(self) inotify_watch_state: InotifyWatchState
}

pub(self) struct InotifyWatchState {
    pub(self) wd_to_metadata: HashMap<WatchDescriptor, WatchMetadata>,
    pub(self) path_to_wd: HashMap<PathBuf, WatchDescriptor>
}

#[derive(Clone)]
pub(super) struct WatchMetadata {
    pub(super) wd: WatchDescriptor,
    pub(super) ws_root_path: PathBuf,
    pub(super) relative_path: Option<PathBuf>,
    pub(super) path: PathBuf,
    pub(super) child_watches: Vec<WatchDescriptor>
}

impl<'a> MonitorState<'a> {
    pub(super) fn default(ws_info: &'a WorkspaceInfo) -> Self {
        MonitorState {
            workspace_info: ws_info,
            inotify_watch_state: InotifyWatchState::default()
        }
    }

    pub(super) fn reset_state(&mut self) {
        self.inotify_watch_state.clear_state();
    }

    pub(super) fn contains_wd(&self, wd: &WatchDescriptor) -> Result<bool, Error> {
        self.inotify_watch_state.contains_wd(wd)
    }

    pub(super) fn add_watch_metadata(&mut self, path: &PathBuf, wd: &WatchDescriptor, md: &WatchMetadata) {
        self.inotify_watch_state.add_watch_metadata(path, wd, md)
    }

    pub(super) fn rm_watch_metadata(&mut self, wd: &WatchDescriptor) -> Result<(), Error> {
        self.inotify_watch_state.rm_watch_metadata(&wd)
    }

    pub(super) fn get_wd(&self, path: &PathBuf) -> Option<&WatchDescriptor> {
        self.inotify_watch_state.path_to_wd.get(path)
    }

    pub(super) fn get_metadata(&self, wd: &WatchDescriptor) -> Option<&WatchMetadata> {
        self.inotify_watch_state.wd_to_metadata.get(wd)
    }

    pub(super) fn get_metadata_mut(&mut self, wd: &WatchDescriptor) -> Option<&mut WatchMetadata> {
        self.inotify_watch_state.wd_to_metadata.get_mut(wd)
    }

    pub(super) fn get_metadata_from_path(&self, path: &PathBuf) -> Option<&WatchMetadata> {
        let watch_descriptor = self.get_wd(path);

        if let Some(wd) = watch_descriptor {
            return self.get_metadata(wd);
        }

        None
    }

    pub(super) fn get_metadata_mut_from_path(&mut self, path: &PathBuf) -> Option<&mut WatchMetadata> {
        let watch_descriptor = self.get_wd(path).cloned();

        if let Some(wd) = watch_descriptor {
            return self.get_metadata_mut(&wd);
        }

        None
    }
}

impl InotifyWatchState {
    pub(self) fn default() -> Self {
        InotifyWatchState {
            wd_to_metadata: HashMap::new(),
            path_to_wd: HashMap::new()
        }
    }

    pub(self) fn clear_state(&mut self) {
        self.wd_to_metadata.clear();
        self.path_to_wd.clear();
    }

    pub(self) fn contains_wd(&self, wd: &WatchDescriptor) -> Result<bool, Error> {

        match self.wd_to_metadata.get(wd) {
            Some(metadata) => {
                if !self.path_to_wd.contains_key(&metadata.path) {
                    return Err(Error::InconsistentState("Path-to-wd mapping missing".to_string()));
                }
                Ok(true)
            },
            None => {
                if self.path_to_wd.values().any(|v| v == wd) {
                    return Err(Error::InconsistentState("wd-to-metadata mapping missing".to_string()));
                }
                Ok(false)
            }
        }
    }

    pub(self) fn add_watch_metadata(&mut self, path: &PathBuf, wd: &WatchDescriptor, md: &WatchMetadata) {
        self.wd_to_metadata.insert(wd.clone(), md.clone());
        self.path_to_wd.insert(path.clone(), wd.clone());
    }

    pub(self) fn rm_watch_metadata(&mut self, wd: &WatchDescriptor) -> Result<(), Error> {
        let contained = self.contains_wd(wd)?;

        if !contained {
            return Err(Error::NotFound(format!("No metadata stored for wd '{:?}'", wd)));
        }

        let metadata = self.wd_to_metadata.remove(wd).unwrap();
        let _ = self.path_to_wd.remove(&metadata.path);

        Ok(())
    }
}

impl WatchMetadata {
    pub(super) fn initialize(
        wd: &WatchDescriptor,
        ws_root_path: &PathBuf,
        relative_path: Option<&PathBuf>,
        path: &PathBuf
    ) -> Self {
        WatchMetadata {
            wd: wd.clone(),
            ws_root_path: ws_root_path.clone(),
            relative_path: relative_path.cloned(),
            path: path.clone(),
            child_watches: vec![]
        }
    }
}

impl Display for WatchMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Wd ({:?}), Relative Path ({:?}), Path ({:?}), Nr. of Child Watches ({}), Child Watches ({:?})",
            self.wd,
            self.relative_path,
            self.path,
            self.child_watches.len(),
            self.child_watches
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound(msg) => {
                write!(f, "[Not_Found]: {}", msg)
            },
            Error::InconsistentState(msg) => {
                write!(f, "[Inconsistent_State]: {}", msg)
            },
            Error::Error(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}
