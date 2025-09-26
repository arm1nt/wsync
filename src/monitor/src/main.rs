use std::io;
use std::io::{BufReader, Read, Stdin};
use log::debug;
use serde::de::DeserializeOwned;
use serde_json::{Deserializer, StreamDeserializer};
use serde_json::de::IoRead;
use util::log::setup_logging;
use crate::models::{Error, WorkspaceInfo};
use crate::sync::synchronize_workspace;
use crate::util::error_exit;

mod util;
mod linux;
mod macos;
mod sync;
mod models;

fn get_json_deserializer<R: Read, T: DeserializeOwned>(reader: R) -> StreamDeserializer<'static, IoRead<BufReader<R>>, T> {
    let r = BufReader::new(reader);
    Deserializer::from_reader(r).into_iter::<T>()
}

fn validate_workspace_info(workspace_info: &WorkspaceInfo) -> Result<(), Error> {
    if !workspace_info.local_path.exists() {
        return Err(Error::new(
            format!("Workspace '{}' at '{:?}' does not exist!", workspace_info.name, workspace_info.local_path)
        ));
    }

    if !workspace_info.local_path.is_dir() {
        return Err(Error::new(
            format!("'{:?}' does not reference a workspace directory!", workspace_info.local_path)
        ));
    }

    Ok(())
}

fn get_workspace_information() -> Result<WorkspaceInfo, Error> {
    debug!("Attempting to get and parse workspace information....");

    let mut deserializer = get_json_deserializer::<Stdin, WorkspaceInfo>(io::stdin());
    let data = deserializer.next();

    let data = data
        .ok_or_else(|| Error::new("No input workspace information found"))?
        .map_err(|e| Error::new(format!("Unable to read input workspace information: {e}")))?
        ;

    debug!("Successfully got workspace information!");

    validate_workspace_info(&data)?;
    Ok(data)
}

fn main() {
    setup_logging();

    let workspace: WorkspaceInfo = get_workspace_information().unwrap_or_else(|e| {
        error_exit(Some(format!("{e}")));
    });

    // To account for possible workspace changes that happened while the monitor was inactive, sync
    // the entire ws with all remote workspaces.
    let _ = synchronize_workspace(&workspace, None).unwrap_or_else(|e| {
        error_exit(Some(format!("Failed initial sync of workspace with remote systems: {e:?}")))
    });

    if cfg!(target_os = "linux") {
        linux::run_fs_listener(workspace);
    } else if cfg!(target_os = "macos") {
        macos::run_fs_listener(workspace);
    } else {
        panic!("OS not supported (yet)");
    }
}
