use std::fmt::format;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::ptr::read;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;
use daemon_interface::{Command, Response, WorkspaceInfoRequest};
use crate::types::daemon_state::DaemonState;

struct HandlerError {
    error_msg_to_log: String,
    error_msg_for_user: String
}

fn get_buffered_reader(stream: &UnixStream) -> Result<BufReader<UnixStream>, String> {
    let reader = BufReader::new(stream.try_clone().map_err(|e| {
        format!("Unable to create buffered reader needed to read client data: {e:?}").to_string()
    })?);

    Ok(reader)
}

fn get_buffered_writer(stream: &UnixStream) -> Result<BufWriter<UnixStream>, String> {
    let writer = BufWriter::new(stream.try_clone().map_err(|e| {
        format!("Unable to create buffered writer needed to send data to the client: {e:?}").to_string()
    })?);

    Ok(writer)
}

fn read_line(mut reader: &mut BufReader<UnixStream>) -> Result<String, String> {
    let mut buffer: String = String::new();

    match reader.read_line(&mut buffer) {
        Ok(read_bytes) => {
            if read_bytes == 0 {
                return Err("Connection was closed before being able to read the req. data sent by the client".to_string());
            }
            Ok(buffer)
        },
        Err(e) => {
            Err(format!("Unable to read the req. data sent by the client: {:?}", e.to_string()))
        }
    }
}

fn get_request_json<T: DeserializeOwned>(mut reader: &mut BufReader<UnixStream>) -> Result<T, String> {
    let data = read_line(&mut reader)?;

    let value: T = serde_json::from_str(data.trim()).map_err(|e| {
        format!("Unable to deserialize received req. data '{data}: {e:?}'")
    })?;

    Ok(value)
}

fn send_response(mut writer: &mut BufWriter<UnixStream>, response: &Response) -> Result<(), String> {
    let json =  match serde_json::to_string(response) {
        Ok(json) => json,
        Err(e) => return Err(format!("Unable to serialize '{:?}': {e:?}", response))
    };

    writeln!(writer, "{json}").map_err(|e| {
        format!("Unable to write {json} to socket: {e:?}")
    })?;

    writer.flush().map_err(|e| { format!("Unable to send response to client: {e:?}") })?;

    Ok(())
}

fn get_command(mut reader: &mut BufReader<UnixStream>) -> Result<Command, String> {
    let rcv_command_str = read_line(&mut reader)?;

    match Command::from_str(rcv_command_str.trim()) {
        Ok(command) => Ok(command),
        Err(e) => Err(format!("Received invalid command '{}': {e:?}", rcv_command_str.trim()))
    }
}

pub(crate) fn handle_request(req_id: Uuid, mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) {
    let start = Instant::now();
    info!("[{req_id}] BEGIN - Start handling request ...");

    let mut reader = match get_buffered_reader(&stream) {
        Ok(reader) => reader,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    let mut writer = match get_buffered_writer(&stream) {
        Ok(writer) => writer,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    let command: Command = match get_command(&mut reader) {
        Ok(command) => command,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            let _ = send_response(&mut writer, &Response::error(Some(e)));
            info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    let command_handler_result = match command {
        Command::WorkspaceInfo => handle_workspace_info_cmd(req_id, reader, &mut writer, state),
        Command::ListWorkspaces => handle_list_workspaces_cmd(req_id, &mut writer, state),
        Command::ListWorkspaceInfo => handle_list_workspace_info_cmd(req_id, reader, &mut writer, state),
        Command::AddWorkspace => handle_add_workspace_cmd(req_id, reader, &mut writer, state),
        Command::RemoveWorkspace => handle_remove_workspace_cmd(req_id, reader, &mut writer, state),
        Command::AttachRemoteWorkspace => handle_attach_remote_workspace_cmd(req_id, reader, &mut writer, state),
        Command::DetachRemoteWorkspace => handle_detach_remote_workspace_cmd(req_id, reader, &mut writer, state)
    };

    if command_handler_result.is_err() {
        let error = command_handler_result.err().unwrap();
        warn!("[{req_id}] Cannot handle request: {error}");
        let _ = send_response(&mut writer, &Response::error(Some(error)));
    }

    info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
    let _ = stream.shutdown(Shutdown::Both);
}

fn handle_workspace_info_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling workspace info command");

    let data: WorkspaceInfoRequest = get_request_json(&mut reader)?;

    let daemon_state = state.lock().unwrap();

    let ws_config_entry = daemon_state.ws_config
        .find_by_name(data.name.clone())
        .map_err(|e| { e.msg })?;

    drop(daemon_state);

    let response: Response = match ws_config_entry {
        Some(entry) => Response::map_to_success(entry)?,
        None => Response::not_found(Some(format!("No workspace with the name '{}' exists", data.name)))
    };

    send_response(&mut writer, &response)?;

    Ok(())
}

fn handle_list_workspaces_cmd(
    req_id: Uuid,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling list workspaces command");

    let daemon_state = state.lock().unwrap();

    let ws_config_entries: Vec<(String, PathBuf)> = daemon_state.ws_config
        .parse()
        .map_err(|e| { e.msg })?
        .iter()
        .map(|entry| {
            (entry.name.clone(), entry.local_path.clone())
        })
        .collect();

    drop(daemon_state);

    let response = Response::map_to_success(ws_config_entries)?;
    send_response(&mut writer, &response)?;

    Ok(())
}

fn handle_list_workspace_info_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling list workspace info command");
    todo!()
}

fn handle_add_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling add workspace command");
    todo!()
}

fn handle_remove_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling remove workspace command");
    todo!()
}

fn handle_attach_remote_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling attach remote workspace command");
    todo!()
}

fn handle_detach_remote_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: &mut BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), String> {
    debug!("[{req_id}] Handling detach remote workspace command");
    todo!()
}
