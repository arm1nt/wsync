use std::fmt::{format, Debug};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::{Serialize};
use serde_json::Deserializer;
use uuid::Uuid;
use daemon_interface::{AddWorkspaceRequest, AttachRemoteWorkspaceRequest, Command, RemoveWorkspaceRequest, Response, WorkspaceInfoRequest};
use crate::types::daemon_state::DaemonState;
use crate::types::{ConnectionInfo, RemoteWorkspace, WorkspaceInformation};
use crate::types::errors::{ClientError, HandlerError};
use crate::workspace_config::WorkspaceConfiguration;

struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>
}

impl Client {
    fn new(stream: UnixStream) -> Result<Self, ClientError> {
        let r = stream.try_clone()?;
        let w = stream;
        Ok( Self { reader: BufReader::new(r), writer: BufWriter::new(w)} )
    }

    fn read_line(&mut self) -> Result<String, ClientError> {
        let mut buf = String::new();
        let bytes_read = self.reader.read_line(&mut buf)?;

        if bytes_read == 0 {
            return Err(ClientError::Protocol("Connection closed before reading request data"));
        }

        Ok(buf)
    }

    fn read_json<T: DeserializeOwned>(&mut self) -> Result<T, ClientError> {
        let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<T>();
        let data = stream.next();

        if data.is_none() {
            return Err(ClientError::Protocol("Missing command data"));
        }

        Ok(data.unwrap()?)
    }

    fn write_json<T: Serialize>(&mut self, data: &T) -> Result<(), ClientError> {
        serde_json::to_writer(&mut self.writer, data)?;
        self.writer.flush()?;
        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.writer.get_ref().shutdown(Shutdown::Both);
    }
}

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

    let mut command = match get_command(&mut client) {
        Ok(command) => command,
        Err(e) => {
            if let Some(msg) = e.log { warn!("{}", msg)}
            let _= client.write_json(&Response::error(e.client));
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
        if let Some(msg) = err.log { warn!("{}", msg)}
        let _ = client.write_json(&Response::error(err.client));
    }

    info!("[{req_id}] END - Done handling request (elapsed time: {:?})", Instant::now() - start);
    client.shutdown();
}

fn get_command(client: &mut Client) -> Result<Command, HandlerError> {
    let rcvd_raw_command = client.read_line().map_err(|e| {
        HandlerError::both(
            format!("Failed to read command data: {e}"),
            "Failed to read command data"
        )
    })?;

    match Command::from_str(rcvd_raw_command.trim()) {
        Ok(command) => Ok(command),
        Err(e) => Err(HandlerError::both(
            format!("Received invalid command '{}': {e}", rcvd_raw_command.trim()),
            format!("Received invalid command '{}'", rcvd_raw_command.trim())
        ))
    }
}

fn handle_workspace_info_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'workspace_info' command...");
    todo!()
}

fn handle_list_workspaces_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'list_workspaces' command...");
    todo!()
}

fn handle_list_workspace_info_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'list_workspace_info' command...");
    todo!()
}

fn handle_add_workspace_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'add_workspace' command...");
    todo!()
}

fn handle_remove_workspace_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'remove_workspace' command...");
    todo!()
}

fn handle_attach_remote_workspace_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'attach_remote_workspace' command...");
    todo!()
}

fn handle_detach_remote_workspace_cmd(
    req_id: Uuid,
    client: &mut Client,
    state: Arc<Mutex<DaemonState>>
) -> Result<(), HandlerError> {
    debug!("[{req_id}] Handling 'detach_remote_workspace' command...");
    todo!()
}
