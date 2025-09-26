use std::fs;
use std::fs::{DirEntry, FileType};
use std::path::PathBuf;
use crate::models::Error;

type Result<T> = std::result::Result<T, Error>;

pub(crate) fn concat_paths(base_path: Option<&PathBuf>, relative_path: Option<&PathBuf>) -> Result<PathBuf> {
    match (base_path, relative_path) {
        (None, None) => Err(Error::new("Neither base nor relative path are specified!")),
        (Some(base), None) => Ok(base.clone()),
        (None, Some(relative)) => Ok(relative.clone()),
        (Some(base), Some(relative)) => Ok(base.join(relative))
    }
}

pub(crate) fn get_subdir_entries(dir: &PathBuf) -> Result<Vec<DirEntry>> {

    if !dir.exists() {
        return Err(Error::new(format!("'{:?}' does not exist", dir)));
    }

    if !dir.is_dir() {
        return Err(Error::new(format!("'{:?}' does not refer to a directory!", dir)));
    }

    let mut subdirs = vec![];

    for entry_res in fs::read_dir(&dir)? {
        let entry = entry_res.map_err(|e| Error::new(format!("{e}")))?;
        let file_type = entry.file_type().map_err(|e| {
            Error::new(format!("Unable to get file type information of '{:?}': {e}", entry))
        })?;

        if !accepted_filetype(&file_type) {
            continue
        }

        subdirs.push(entry);
    }

    Ok(subdirs)
}

pub(crate) fn get_subdir_names(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let subdir_entries = get_subdir_entries(dir)?;

    let subdir_names = subdir_entries
        .into_iter()
        .map(|entry| PathBuf::from(entry.file_name()))
        .collect::<Vec<PathBuf>>();

    Ok(subdir_names)
}

#[inline(always)]
pub(crate) fn accepted_filetype(path: &FileType) -> bool {
    path.is_dir()
}

pub(crate) fn strip_ws_root_prefix(ws_root: &PathBuf, abs_path: &PathBuf) -> Result<Option<PathBuf>> {

    if !abs_path.is_absolute() {
        return Err(Error::new(format!("'{:?}' is not absolute!", abs_path)));
    }

    if !abs_path.starts_with(ws_root) {
        return Err(Error::new(format!("'{:?}' is not in the workspace", abs_path)));
    }

    if ws_root == abs_path {
        return Ok(None);
    }

    match abs_path.strip_prefix(ws_root) {
        Ok(stripped) => Ok(Some(PathBuf::from(stripped))),
        Err(e) => {
            Err(Error::new(format!("Unable to remove prefix: {:?}", e)))
        }
    }
}
