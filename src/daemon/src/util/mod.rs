pub(crate) mod log;

use std::process;
use ::log::error;

pub(crate) fn error_exit<T: AsRef<str>>(msg: Option<T>) -> ! {
    if msg.is_some() {
        error!("{}", msg.unwrap().as_ref())
    }
    error!("Terminating wsync daemon!");
    process::exit(1);
}
