use std::sync::{Arc, Mutex};
use std::{process, thread};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{info, LevelFilter};
use crate::types::daemon_state::DaemonState;
use crate::types::socket::UnlinkingListener;
use crate::util::error_exit;

mod workspace_config;
mod util;
mod types;
mod monitor_manager;

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
    todo!()
}

fn main() {
    println!("Starting wsync daemon...");
    setup_logging();
}
