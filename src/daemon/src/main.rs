use std::sync::{Arc, Mutex};
use std::{process, thread};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{error, info, LevelFilter};
use crate::handlers::handle_request;
use crate::types::daemon_state::DaemonState;
use crate::types::socket::UnlinkingListener;
use crate::util::error_exit;

mod workspace_config;
mod util;
mod types;
mod monitor_manager;
mod handlers;

const MAX_CONSECUTIVE_CONNECTION_FAILURES: i32 = 10;

fn setup_logging() {
    let stdout = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(PatternEncoder::new("{h({d(%Y-%m-%d %H:%M:%S)} - [{l}]: {m}{n})}")))
        .build();

    let appender = Appender::builder().build("stdout", Box::new(stdout));

    let config = Config::builder()
        .appender(appender)
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap_or_else(|e| {
            eprintln!("An error occurred while initializing the logging infrastructure: {e:?}");
            process::exit(1);
        });

    log4rs::init_config(config).unwrap_or_else(|e| {
        eprintln!("An error occurred while initializing the logging infrastructure: {e:?}");
        process::exit(1);
    });

    info!("Initialized logging framework");
}

fn server_loop(state: Arc<Mutex<DaemonState>>) {
    info!("Starting wsync daemon server loop...");

    let mut consecutive_connection_failures = 0;

    let listener: UnlinkingListener = match UnlinkingListener::bind() {
        Ok(listener) => listener,
        Err(e) => {
            error_exit(Some(format!("Error creating socket for daemon server loop: {e:?}")));
        }
    };

    for stream in listener.listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("Successfully established connection with a client");
                consecutive_connection_failures = 0;

                let cloned_state = Arc::clone(&state);
                let _handle = thread::spawn(move || { handle_request(stream, cloned_state) });
            },
            Err(e) => {
                error!("Failed to establish connection with a client: {e:?}");
                consecutive_connection_failures += 1;

                if consecutive_connection_failures > MAX_CONSECUTIVE_CONNECTION_FAILURES {
                    drop(listener); // Manually drop so that the socket file gets removed
                    error_exit(
                        Some(
                            format!(
                                "Daemon failed {MAX_CONSECUTIVE_CONNECTION_FAILURES} consecutive times to establish a connection with a client"
                            )
                        )
                    );
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

    server_loop(state);
}
