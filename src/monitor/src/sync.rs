use std::path::PathBuf;
use log::{debug, error, warn};
use std::fmt::Write;
use std::process::{Command, ExitStatus, Stdio};
use crate::models::{ConnectionInfo, RemoteWorkspace, WorkspaceInfo};
use crate::util::error_exit;
use crate::util::fs::concat_paths;

#[derive(Debug)]
pub(super) enum Error {
    RemoteSystemError(String),
    LocalError(String)
}
type Result<T> = std::result::Result<T, Error>;

fn pathbuf_to_string(path: PathBuf) -> Result<String> {
    path.to_str()
        .ok_or(Error::LocalError(format!("Error stringifying '{:?}'", path)))
        .map(|val| val.to_string())
}

fn to_dir_arg_path(base: &PathBuf, relative_path: Option<&PathBuf>) -> Result<String> {
    let dir_path = concat_paths(Some(base), relative_path).map_err(|e| {
        Error::LocalError(format!("Unable to concat '{:?}' and '{:?}': {e}", base, relative_path))
    })?;

    pathbuf_to_string(dir_path)
}

fn get_remote_dir_arg_ssh(
    relative_path: Option<&PathBuf>,
    remote_workspace: &RemoteWorkspace
) -> Result<String> {
    let connection_info = &remote_workspace.connection_info;
    let mut arg = String::new();

    if let ConnectionInfo::Ssh { username: Some(username), .. } = connection_info {
        write!(&mut arg, "{}@", username).map_err(|e| {
            Error::LocalError(format!("Error appending username to arg string: {e}"))
        })?;
    }

    if let ConnectionInfo::Ssh { host, .. } = connection_info {
        write!(&mut arg, "{}", host).map_err(|e| {
            Error::LocalError(format!("Error appending host to arg string: {e}"))
        })?;
    }

    let dir_path = to_dir_arg_path(&remote_workspace.remote_path, relative_path)?;
    write!(&mut arg, ":{}", dir_path).map_err(|e| {
        Error::LocalError(format!("Unable to append target path to arg string: {e}"))
    })?;

    Ok(arg)
}

fn get_remote_dir_arg_host_alias(
    relative_path: Option<&PathBuf>,
    remote_workspace: &RemoteWorkspace
) -> Result<String> {
    let dir_path = to_dir_arg_path(&remote_workspace.remote_path, relative_path)?;

    let alias = match &remote_workspace.connection_info {
        ConnectionInfo::HostAlias { host_alias } => host_alias,
        _ => error_exit(None) // Unreachable
    };

    Ok(format!("{}:{}", alias, dir_path))
}

fn get_remote_dir_arg_rsync_daemon(
    relative_path: Option<&PathBuf>,
    remote_workspace: &RemoteWorkspace
) -> Result<String> {
    let connection_info = &remote_workspace.connection_info;
    let mut arg = String::from("rsync://");

    if let ConnectionInfo::RsyncDaemon { username: Some(username), .. } = connection_info {
        write!(&mut arg, "{}@", username).map_err(|e| {
            Error::LocalError(format!("Error appending username to arg string: {e}"))
        })?;
    }

    if let ConnectionInfo::RsyncDaemon { host, .. } = connection_info {
        write!(&mut arg, "{}", host).map_err(|e| {
            Error::LocalError(format!("Error appending host to arg string: {e}"))
        })?;
    }

    if let ConnectionInfo::RsyncDaemon { port: Some(port), .. } = connection_info {
        write!(&mut arg, ":{}", port).map_err(|e| {
            Error::LocalError(format!("Error appending port to arg string: {e}"))
        })?;
    }

    let dir_path = to_dir_arg_path(&remote_workspace.remote_path, relative_path)?;
    write!(&mut arg, "{}", dir_path).map_err(|e| {
        Error::LocalError(format!("Unable to append target path to arg string: {e}"))
    })?;

    Ok(arg)
}

fn get_target_dir_arg(relative_path: Option<&PathBuf>, remote_workspace: &RemoteWorkspace) -> Result<String> {

    match remote_workspace.connection_info {
        ConnectionInfo::HostAlias { .. } => {
            get_remote_dir_arg_host_alias(relative_path, remote_workspace)
        },
        ConnectionInfo::Ssh { .. } => {
            get_remote_dir_arg_ssh(relative_path, remote_workspace)
        },
        ConnectionInfo::RsyncDaemon { .. } => {
            get_remote_dir_arg_rsync_daemon(relative_path, remote_workspace)
        }
    }
}

fn get_source_dir_arg(ws_root_path: &PathBuf, relative_path: Option<&PathBuf>) -> Result<String> {
    let mut src_dir_arg = to_dir_arg_path(ws_root_path, relative_path)?;

    // The src dir arg must end with a trailing '/' symbol to avoid that rsync creates an additional
    // directory level at the remote workspace.
    if !src_dir_arg.ends_with("/") {
        src_dir_arg.push('/');
     }

    Ok(src_dir_arg)
}

fn get_remote_shell_args_ssh(remote_workspace: &RemoteWorkspace) -> Result<String> {

    let mut arg = String::from("-e \"ssh ");

    if let ConnectionInfo::Ssh { port: Some(port), .. } = &remote_workspace.connection_info {
        write!(&mut arg, "-p {}", port).map_err(|e| {
            Error::LocalError(format!("Error appending port to remote shell arg: {e}"))
        })?;
    }

    if let ConnectionInfo::Ssh { identity_file: Some(identity_file), .. } = &remote_workspace.connection_info {
        let dir_path = pathbuf_to_string(identity_file.clone())?;
        write!(&mut arg, " -i {}", dir_path).map_err(|e| {
            Error::LocalError(format!("Error appending identity file to remote shell arg: {e}"))
        })?;
    }

    arg.push('\"');

    Ok(arg)
}

fn get_rsync_arguments(
    ws_root_path: &PathBuf,
    relative_path: Option<&PathBuf>,
    remote_workspace: &RemoteWorkspace
) -> Result<Vec<String>> {

    let mut args: Vec<String> = vec![];

    args.push(String::from("-azq"));
    args.push(String::from("--delete"));

    // Add extra remote shell arguments
    match remote_workspace.connection_info {
        ConnectionInfo::Ssh { .. } => {
            let remote_shell_arg = get_remote_shell_args_ssh(remote_workspace)?;
            args.push(remote_shell_arg);
        },
        _ => {}
    }

    let src = get_source_dir_arg(ws_root_path, relative_path)?;
    args.push(src);

    let dst = get_target_dir_arg(relative_path, remote_workspace)?;
    args.push(dst);

    Ok(args)
}

fn execute_rsync_command(args: Vec<String>) -> Result<()> {
    debug!("Attempting to sync with args: '{:?}'", args);

    let mut rsync_output = match Command::new("rsync").args(&args).output() {
        Ok(output) => output,
        Err(error) => {
            return Err(Error::LocalError(format!("Unable to run 'rsync': {error}")));
        }
    };

    if !rsync_output.stdout.is_empty() {
        let stringified_stdout = String::from_utf8_lossy(&rsync_output.stdout);
        warn!("{}", stringified_stdout);
    }

    if !rsync_output.stderr.is_empty() {
        let stringified_stderr = String::from_utf8_lossy(&rsync_output.stderr);
        error!("{}", stringified_stderr);
    }

    if !rsync_output.status.success() {
        return Err(Error::RemoteSystemError(
            format!("'rsync' returned status code '{}'", rsync_output.status)
        ));
    }

    Ok(())
}

fn synchronize_remote_workspace(
    ws_root_path: &PathBuf,
    relative_path: Option<&PathBuf>,
    remote_workspace: &RemoteWorkspace
) -> Result<()> {
    let args = get_rsync_arguments(ws_root_path, relative_path, remote_workspace)?;

    match execute_rsync_command(args) {
        Ok(_) => {},
        Err(Error::RemoteSystemError(_)) if relative_path.is_some() => {
            // Attempt to sync the workspace starting from the ws root, since its possible that
            //  (parts) of the remote ws were deleted/moved/etc., causing rsync to fail.
            warn!(
                "Syncing with '{}' failed. Attempting to sync from ws root to re-build the dir tree...",
                remote_workspace.name
            );

            let args = get_rsync_arguments(ws_root_path, None, remote_workspace)?;
            return execute_rsync_command(args);
        },
        Err(e) => return Err(e)
    }

    Ok(())
}

pub(crate) fn synchronize_workspace(
    workspace_info: &WorkspaceInfo,
    relative_path: Option<&PathBuf>
) -> Result<()> {

    for remote_workspace in workspace_info.remote_workspaces.iter() {

        let sync_result = synchronize_remote_workspace(
            &workspace_info.local_path,
            relative_path,
            remote_workspace
        );

        match sync_result {
            Err(Error::RemoteSystemError(msg)) => {
                warn!("Failed to sync with '{}': {msg}", remote_workspace.name);
            },
            Err(e @ Error::LocalError(_)) => {
                return Err(e)
            }
            _ => {}
        }
    }

    Ok(())
}
