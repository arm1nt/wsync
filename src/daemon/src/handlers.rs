use std::io::{read_to_string, BufRead, BufReader, BufWriter, Read, Write};
use std::net::Shutdown;
use std::os::linux::raw::stat;
use std::os::unix::net::UnixStream;
use std::ptr::read;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;
use daemon_interface::{Command, Response, WorkspaceInfoRequest};
use crate::server_loop;
use crate::types::daemon_state::DaemonState;

fn get_buffered_reader(stream: &mut UnixStream) -> Result<BufReader<UnixStream>, String> {
    let reader = BufReader::new(stream.try_clone().map_err(|e| {
        format!("Unable to create buffered reader needed to read client data: {e:?}").to_string()
    })?);

    Ok(reader)
}

fn get_buffered_writer(stream: &mut UnixStream) -> Result<BufWriter<UnixStream>, String> {
    let writer = BufWriter::new(stream.try_clone().map_err(|e| {
        format!("Unable to create buffered writer needed to send data to the client: {e:?}").to_string()
    })?);

    Ok(writer)
}

fn read_line(mut reader: &mut BufReader<UnixStream>) -> Result<String, String> {
    let mut data = String::new();

    match reader.read_line(&mut data) {
        Ok(read_bytes) => {
            if read_bytes == 0 {
                return Err("The connection was closed before being able to read the req. data sent by the client".to_string())
            }

            Ok(data)
        },
        Err(e) => {
            Err(format!("Unable to read the req. data sent by the client: {:?}", e.to_string()))
        }
    }
}

fn get_client_command(mut reader: &mut BufReader<UnixStream>) -> Result<Command, String> {
    let rcv_command_str = match read_line(&mut reader) {
        Ok(cmd_str) => cmd_str,
        Err(e) => { return Err(e) }
    };

    match Command::from_str(rcv_command_str.trim()) {
        Ok(command) => Ok(command),
        Err(e) => Err(format!("Received invalid command '{}': {e:?}", rcv_command_str.trim()))
    }
}

fn get_req_json<T: DeserializeOwned>(mut reader: &mut BufReader<UnixStream>) -> Result<T, String> {
    let data = match read_line(&mut reader) {
        Ok(data) => data,
        Err(e) => { return Err(e); }
    };

    let val: T = serde_json::from_str(data.as_str()).map_err(|e| {
        format!("Unable to deserialize received req. data '{data}'")
    })?;

    Ok(val)
}

pub fn handle_request(req_id: Uuid, mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) {
    let start = Instant::now();
    info!("[{req_id}] Start handling request...");

    let mut reader = match get_buffered_reader(&mut stream) {
        Ok(reader) => reader,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            info!("[{req_id}] Done handling request (elapsed time: {:?})", Instant::now() - start);

            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    let mut writer = match get_buffered_writer(&mut stream) {
        Ok(writer) => writer,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            info!("[{req_id}] Done handling request (elapsed time: {:?})", Instant::now() - start);

            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    let command = match get_client_command(&mut reader) {
        Ok(command) => command,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");

            match serde_json::to_string(&Response::error(Some(e))) {
                Ok(json) => {
                    let _ = writeln!(writer, "{json}");
                    let _ = writer.flush();
                },
                Err(_) => {}
            }

            info!("[{req_id}] Done handling request (elapsed time: {:?})", Instant::now() - start);

            let _ = stream.shutdown(Shutdown::Both);
            return;
        }
    };

    match command {
        Command::WorkspaceInfo => handle_workspace_info_cmd(req_id, reader, writer, state),
        Command::ListWorkspaces => handle_list_workspaces_cmd(req_id, reader, writer, state),
        Command::ListWorkspaceInfo => handle_list_workspace_info_cmd(req_id, reader, writer, state),
        Command::AddWorkspace => handle_add_workspace_cmd(req_id, reader, writer, state),
        Command::RemoveWorkspace => handle_remove_workspace_cmd(req_id, reader, writer, state),
        Command::AttachRemoteWorkspace => handle_attach_remote_workspace_cmd(req_id, reader, writer, state),
        Command::DetachRemoteWorkspace => handle_detach_remote_workspace_cmd(req_id, reader, writer, state)
    }

    info!("[{req_id}] Done handling request (elapsed time: {:?})", Instant::now() - start);
    let _ = stream.shutdown(Shutdown::Both);
}

fn handle_workspace_info_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    debug!("Handling workspace info request with request id '{req_id}'");

    let data: WorkspaceInfoRequest = match get_req_json(&mut reader) {
        Ok(data) => data,
        Err(e) => {
            warn!("[{req_id}] Cannot handle request: {e:?}");
            return;
        }
    };

    todo!()
}

fn handle_list_workspaces_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}

fn handle_list_workspace_info_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}

fn handle_add_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}

fn handle_remove_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}

fn handle_attach_remote_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}

fn handle_detach_remote_workspace_cmd(
    req_id: Uuid,
    mut reader: BufReader<UnixStream>,
    mut writer: BufWriter<UnixStream>,
    state: Arc<Mutex<DaemonState>>
) {
    todo!()
}
