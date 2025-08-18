use std::fmt::Display;
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{debug, info, warn};
use serde::Serialize;
use uuid::Uuid;
use client::client::Client;
use daemon_interface::request::{
    AddWorkspaceRequest,
    AttachRemoteWorkspaceRequest,
    Command,
    CommandRequest,
    DetachRemoteWorkspaceRequest,
    RemoveWorkspaceRequest,
    WorkspaceInfoRequest
};
use daemon_interface::response::{DefaultResponse, Response, ResponsePayload};
use daemon_interface::response::ErrorPayload::Message;
use crate::daemon_state::DaemonState;
use crate::domain::errors::WsConfigError;
use crate::domain::models::{RemoteWorkspace, WorkspaceInformation};
use crate::handlers::errors::HandlerError;
use crate::handlers::mappers::domain_to_interface::{
    to_list_workspace_info_response,
    to_list_workspaces_response,
    to_workspace_info_response
};

pub(crate) fn handle_request(req_id: Uuid, stream: UnixStream, state: Arc<Mutex<DaemonState>>) {
    let start = Instant::now();
    info!("[{req_id}] BEGIN - Start handling request ...");

    let mut client = match Client::new(stream) {
        Ok(client) => client,
        Err(e) => {
            warn!("[{req_id}] {e}");
            info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
            return;
        }
    };

    let command = match get_command(&mut client) {
        Ok(command) => command,
        Err(e) => {
            if let Some(err) = e.log { warn!("[{req_id}] {}", err)}
            if let Some(err) = e.client {
                let response: DefaultResponse = Response::error(Some(Message(err)));
                let _ = client.write_json(&response);
            }
            info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
            client.shutdown();
            return;
        }
    };

    let command_handler_result = match command {
        Command::WorkspaceInfo => handle_workspace_info_cmd(req_id, &mut client, state),
        Command::ListWorkspaces => handle_list_workspaces_cmd(req_id, &mut client, state),
        Command::ListWorkspaceInfo => handle_list_workspace_info_cmd(req_id, &mut client, state),
        Command::AddWorkspace => handle_add_workspace_cmd(req_id, &mut client, state),
        Command::RemoveWorkspace => handle_remove_workspace_cmd(req_id, &mut client, state),
        Command::AttachRemoteWorkspace => handle_attach_remote_workspace_cmd(req_id, &mut client, state),
        Command::DetachRemoteWorkspace => handle_detach_remote_workspace_cmd(req_id, &mut client, state)
    };

    if let Err(err) = command_handler_result {
        if let Some(msg) = err.log { warn!("[{req_id}] {}", msg)}
        if let Some(err) = err.client {
            let response: DefaultResponse = Response::error(Some(Message(err)));
            let _ = client.write_json(&response);
        }
    }

    info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
    client.shutdown();
}

fn get_command(client: &mut Client) -> Result<Command, HandlerError> {
    let raw_client_command: CommandRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Failed to read user command: {e}"),
            "Failed to read command"
        )
    })?;

    match Command::from_str(raw_client_command.command.as_str()) {
        Ok(command) => Ok(command),
        Err(e) => Err(HandlerError::both(
            format!("Received invalid command '{}': {e}", raw_client_command.command.as_str()),
            format!("Received invalid command '{}'", raw_client_command.command.as_str())
        ))
    }
}

fn handle_workspace_info_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'workspace_info' command...");

    let data: WorkspaceInfoRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Unable to read data required to processes the 'workspace_info' command: {e}"),
            "Unable to read data required to process the 'workspace_info' command".to_string()
        )
    })?;

    let guard = state.lock().unwrap();
    let search_result = guard.ws_config.find_by_name(&data.name);
    drop(guard);

    let response = match search_result {
        Some(ws_info) => {
            debug!("[{req_id}] Found a workspace with the name '{}'", data.name);

            let response_data = to_workspace_info_response(ws_info);
            Response::success(Some(ResponsePayload::WorkspaceInfo(response_data)))
        },
        None => {
            debug!("[{req_id}] No workspace with the name '{}' found.", data.name);

            Response::not_found(Some(Message(
                format!("No local workspace with the name '{}' found.", data.name))
            ))
        }
    };

    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_list_workspaces_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'list_workspaces' command...");

    let guard = state.lock().unwrap();
    let ws_entries: Vec<WorkspaceInformation> = guard.ws_config.all();
    drop(guard);

    debug!("[{req_id}] Found #{} workspaces: {:?}", ws_entries.len(), ws_entries);

    let response_data = to_list_workspaces_response(ws_entries);
    let response: DefaultResponse = Response::success(Some(ResponsePayload::ListWorkspaces(response_data)));
    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_list_workspace_info_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'list_workspace_info' command...");

    let guard = state.lock().unwrap();
    let ws_entries = guard.ws_config.all();
    drop(guard);

    debug!("[{req_id}] Found #{} workspaces: {:?}", ws_entries.len(), ws_entries);

    let response_data = to_list_workspace_info_response(ws_entries);
    let response: DefaultResponse = Response::success(Some(ResponsePayload::ListWorkspaceInfo(response_data)));
    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_add_workspace_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'add_workspace' command...");

    let data: AddWorkspaceRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Unable to read data required to processes the 'add_workspace' command: {e}"),
            "Unable to read data required to process the 'add_workspace' command"
        )
    })?;

    let mut guard = state.lock().unwrap();
    let res = guard.ws_config.add_workspace(WorkspaceInformation::from(data.clone()));
    drop(guard);

    let response = match res {
        Ok(()) => {
            debug!("[{req_id}] Successfully added workspace {data:?}");
            Response::success(Some(
                ResponsePayload::AddWorkspace("Successfully added workspace!".to_string()))
            )
        },
        Err(err) => {
            debug!("[{req_id}] Adding workspace {data:?} was not successful");

            match err {
                WsConfigError::Io(e) => {
                    return Err(HandlerError::both(
                        format!("{e}"),
                        "Couldn't add the workspace as an error occurred while trying to modify the workspace configuration file"
                    ));
                },
                WsConfigError::Message(e) => {
                    debug!("[{req_id}] {e}");
                    Response::error(Some(Message(e)))
                }
            }
        }
    };

    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_remove_workspace_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'remove_workspace' command...");

    let data: RemoveWorkspaceRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Unable to read data required to processes the 'remove_workspace' command: {e}"),
            "Unable to read data required to process the 'remove_workspace' command"
        )
    })?;

    let mut guard = state.lock().unwrap();

    let config_res = guard.ws_config.remove_workspace(data.name.clone());
    match config_res {
        Ok(()) => {
            debug!("[{req_id}] Successfully removed workspace '{}' from workspace config file.", &data.name);
        },
        Err(err) => {
            debug!("[{req_id}] Removing workspace '{}' from workspace config file failed.", &data.name);

            return match err {
                WsConfigError::Io(e) => {
                    Err(HandlerError::both(
                        format!("{e}"),
                        "Couldn't remove the workspace as an error occurred while trying to modify the workspace configuration file"
                    ))
                },
                WsConfigError::Message(e) => {
                    debug!("[{req_id}] {e}");
                    let response: DefaultResponse = Response::error(Some(Message(e)));
                    generic_write_json(&mut client, &response)?;
                    Ok(())
                }
            }
        }
    }

    let monitor_manager_res = guard.monitor_manager.terminate_monitor(&data.name);
    match monitor_manager_res {
        Ok(()) => {
            debug!("[{req_id}] Successfully terminated monitor process for workspace '{}'.", &data.name);
        },
        Err(err) => {
            debug!("[{req_id}] Failed to terminate monitor process for workspace '{}'.", &data.name);
            return Err(HandlerError::both(
                format!("{err}"),
                "Couldn't remove the workspace as an error occurred while trying to terminate its monitor process"
            ));
        }
    }

    drop(guard);

    let response: DefaultResponse = Response::success(Some(
        ResponsePayload::RemoveWorkspace("Successfully removed workspace!".to_string()))
    );
    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_attach_remote_workspace_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'attach_remote_workspace' command...");

    let data: AttachRemoteWorkspaceRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Unable to read data required to processes the 'attach_remote_workspace' command: {e}"),
            "Unable to read data required to process the 'attach_remote_workspace' command"
        )
    })?;

    let mut guard = state.lock().unwrap();

    let config_result = guard.ws_config.attach_remote_workspace(
        data.local_workspace_name.clone(),
        RemoteWorkspace::from(data.clone())
    );

    match config_result {
        Ok(()) => {
            debug!(
                "[{req_id}] Successfully attached remote workspace '{}' to '{}' in the workspaces config file",
                data.remote_workspace_name,
                data.local_workspace_name
            );
        },
        Err(err) => {
            debug!(
                "[{req_id}] Failed to attach remote workspace '{}' to '{}' in the workspaces config file",
                data.remote_workspace_name,
                data.local_workspace_name
            );

            return match err {
                WsConfigError::Io(e) => {
                    Err(HandlerError::both(
                        format!("{e}"),
                        format!(
                            "Failed to attach remote workspace '{}' to '{}' because there was an error \
                            while trying to modify the workspaces configuration file",
                            data.remote_workspace_name,
                            data.local_workspace_name
                        )
                    ))
                },
                WsConfigError::Message(e) => {
                    debug!("[{req_id}] {e}");
                    let response: DefaultResponse = Response::error(Some(Message(e)));
                    generic_write_json(&mut client, &response)?;
                    Ok(())
                }
            }
        }
    }

    let updated_workspace: WorkspaceInformation = guard.ws_config
        .find_by_name(&data.local_workspace_name)
        .unwrap();

    let mm_res = guard.monitor_manager.restart_monitor(&updated_workspace);

    drop(guard);

    match mm_res {
        Ok(()) => {
            debug!("[{req_id}] Successfully (re)started the monitor process for workspace '{}'", data.local_workspace_name);
        },
        Err(e) => {
            debug!("[{req_id}] Failed to (re)start the monitor process for workspace '{}'", data.local_workspace_name);

            return Err(HandlerError::both(
                format!("{e}"),
                format!(
                    "(Re)starting the monitor process for workspace '{}' failed, so changes cannot \
                    be synced to the newly attached remote workspace.",
                    data.local_workspace_name
                )
            ));
        }
    }

    let response: DefaultResponse = Response::success(Some(
        ResponsePayload::AttachRemoteWorkspace("Successfully attached remote workspace!".to_string())
    ));
    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn handle_detach_remote_workspace_cmd(
    req_id: Uuid,
    mut client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'detach_remote_workspace' command...");

    let data: DetachRemoteWorkspaceRequest = client.read_json().map_err(|e| {
        HandlerError::both(
            format!("Unable to read data required to processes the 'detach_remote_workspace' command: {e}"),
            "Unable to read data required to process the 'detach_remote_workspace' command"
        )
    })?;

    let mut guard = state.lock().unwrap();

    let config_result = guard.ws_config.detach_remote_workspace(
        data.local_workspace_name.clone(),
        data.remote_workspace_name.clone()
    );

    match config_result {
        Ok(()) => {
            debug!(
                "[{req_id}] Successfully detached remote workspace '{}' from '{}' in the workspaces config file",
                data.remote_workspace_name,
                data.local_workspace_name
            );
        },
        Err(err) => {
            debug!(
                "[{req_id}] Failed to detach remote workspace '{}' from '{}' in the workspaces config file",
                data.remote_workspace_name,
                data.local_workspace_name
            );

            return match err {
                WsConfigError::Io(e) => {
                    Err(HandlerError::both(
                        format!("{e}"),
                        format!(
                            "Failed to detach remote workspace '{}' from '{}' because there was an error \
                            while trying to modify the workspaces configuration file",
                            data.remote_workspace_name,
                            data.local_workspace_name
                        )
                    ))
                },
                WsConfigError::Message(e) => {
                    debug!("[{req_id}] {e}");
                    let response: DefaultResponse = Response::error(Some(Message(e)));
                    generic_write_json(&mut client, &response)?;
                    Ok(())
                }
            }
        }
    }

    let updated_workspace: WorkspaceInformation = guard.ws_config
        .find_by_name(&data.local_workspace_name)
        .unwrap();

    let mm_res = guard.monitor_manager.restart_monitor(&updated_workspace);

    drop(guard);

    match mm_res {
        Ok(()) => {
            debug!("[{req_id}] Successfully (re)started the monitor process for workspace '{}'", data.local_workspace_name);
        },
        Err(e) => {
            debug!("[{req_id}] Failed to (re)start the monitor process for workspace '{}'", data.local_workspace_name);

            return Err(HandlerError::both(
                format!("{e}"),
                format!(
                    "(Re)starting the monitor process for workspace '{}' failed, so changes might \
                    still be synced to the  remote workspace '{}'.",
                    data.local_workspace_name,
                    data.remote_workspace_name
                )
            ));
        }
    }

    let response: DefaultResponse = Response::success(Some(
        ResponsePayload::AttachRemoteWorkspace("Successfully detached remote workspace!".to_string())
    ));
    generic_write_json(&mut client, &response)?;

    Ok(())
}

fn generic_write_json<T: Serialize + Display, E: Serialize + Display>(
    client: &mut Client,
    response: &Response<T, E>
) -> Result<(), HandlerError> {
    client.write_json(response).map_err(|e| {
        HandlerError::both(
            format!("Unable to send response '{response}' to client: {e}"),
            "An error occurred while writing the server response"
        )
    })?;

    Ok(())
}
