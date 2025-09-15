use std::io::{Read, Write};
use std::process;
use std::os::unix::net::UnixStream;
use clap::Parser;
use daemon_client::client::Client;
use daemon_interface::response::DefaultResponse;
use crate::cli::Cli;
use crate::mappers::ClientRequest;

mod cli;
mod mappers;

const WSYNC_DAEMON_CMD_SOCKET: &str = "/tmp/wsync-daemon-cmd.socket";

fn print_banner() {
    println!(r"
          __            __
    |  | /__` \ / |\ | /  `
    |/\| .__/  |  | \| \__,
   ");
}

fn get_client() -> Result<Client, String> {
    let stream = UnixStream::connect(WSYNC_DAEMON_CMD_SOCKET).map_err(|e| {
        format!("Unable to connect to wsync daemon: {e}")
    })?;

    Client::new(stream).map_err(|e| {
        format!("Unable to create client to connect to wsync daemon: {e}")
    })
}

fn handle_request(request: ClientRequest) -> Result<(), String> {
    let mut client = get_client()?;

    client.write_json(&request.command_request).map_err(|e| format!("{e}"))?;

    if let Some(data) = request.command_data {
        client.write_json(&data).map_err(|e| format!("{e}"))?;
    }

    let response: DefaultResponse = client.read_json().map_err(|e|
        format!("Unable to read daemon response: {e}")
    )?;
    println!("{response}");

    Ok(())
}

fn main() {
    print_banner();

    let cli: Cli = Cli::parse();
    let request = ClientRequest::get_client_request(cli).unwrap_or_else(|e| {
        eprintln!("[ERROR] {e}");
        process::exit(1);
    });

    handle_request(request).unwrap_or_else(|e| {
        eprintln!("[ERROR] {e}");
        process::exit(1);
    });
}
