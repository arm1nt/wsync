pub(crate) mod constants;

use std::process;

pub(crate) fn error_exit<T: AsRef<str>>(msg: Option<T>) -> ! {
    if msg.is_some() {
        eprintln!("{}", msg.unwrap().as_ref());
    }
    eprintln!("Terminating wsync daemon!");
    process::exit(1);
}
