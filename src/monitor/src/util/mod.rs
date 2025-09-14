use std::process;
use ::log::error;

pub(crate) mod fs;
pub(crate) mod log;

pub(crate) fn error_exit(msg: Option<String>) -> ! {
    if let Some(msg) = msg {
        error!("{msg}");
    }
    error!("Terminating workspace monitor");
    process::exit(1);
}
