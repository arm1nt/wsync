use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{error, info, warn};
use uuid::Uuid;
use daemon_state::DaemonState;
use crate::domain::socket::UnlinkingListener;
use crate::handlers::handlers::handle_request;
use crate::util::constants::SERVER_SOCKET_PATH_EN_VAR;
use crate::util::error_exit;
use crate::util::log::setup_logging;
use crate::watchdog::watchdog;

mod workspace_config;
mod util;
mod domain;
mod monitor_manager;
mod daemon_state;
mod handlers;
mod watchdog;

const MAX_CONSECUTIVE_CONNECTION_FAILURES: i32 = 10;

fn sigint_handler(shutdown: Arc<AtomicBool>) {
    shutdown.store(true, Ordering::Relaxed);
    let _ = UnixStream::connect(PathBuf::from(env::var(SERVER_SOCKET_PATH_EN_VAR).unwrap()));
}

fn get_server_socket() -> UnlinkingListener {
    let listener: UnlinkingListener = match UnlinkingListener::bind() {
        Ok(listener) => listener,
        Err(e) => {
            error_exit(Some(format!("Error creating socket for daemon server loop: {e:?}")));
        }
    };

    listener
}

fn server_loop(state: Arc<Mutex<DaemonState>>, shutdown: Arc<AtomicBool>) {
    info!("Starting wsync daemon server loop...");

    let mut consecutive_connection_failures = 0;
    let listener: UnlinkingListener = get_server_socket();

    for stream in listener.listener.incoming() {
        if shutdown.load(Ordering::Relaxed) {
            warn!("wsync daemon server socket interrupted. Exiting server loop...");
            break;
        }

        match stream {
            Ok(stream) => {
                consecutive_connection_failures = 0;
                let req_id: Uuid = Uuid::new_v4();
                info!("[{req_id}] Successfully established connection with a client");

                let cloned_state = Arc::clone(&state);
                let _handle = thread::spawn(move || { handle_request(req_id, stream, cloned_state) });
            },
            Err(e) => {
                error!("Failed to establish connection with a client: {e:?}");
                consecutive_connection_failures += 1;

                if consecutive_connection_failures > MAX_CONSECUTIVE_CONNECTION_FAILURES {
                    drop(listener); // Manually drop so that the socket file gets removed
                    error_exit(Some(format!(
                        "Daemon failed {MAX_CONSECUTIVE_CONNECTION_FAILURES} consecutive times to establish a connection with a client"
                    )));
                }
            }
        }
    }

    info!("Terminated wsync daemon server loop");
}

fn main() {
    println!("Starting wsync daemon...");
    setup_logging();

    let state: Arc<Mutex<DaemonState>> = DaemonState::init();
    state.lock().unwrap().restore();

    let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let shutdown_cloned = Arc::clone(&shutdown);

    ctrlc::set_handler(move || { sigint_handler(Arc::clone(&shutdown_cloned)) }).unwrap_or_else(|e| {
        error_exit(Some(format!("Unable to set SIGINT error handler: {e:?}")))
    });

    let watchdog_state_clone = Arc::clone(&state);
    thread::spawn(move || watchdog(watchdog_state_clone) );

    server_loop(state, Arc::clone(&shutdown));
}
