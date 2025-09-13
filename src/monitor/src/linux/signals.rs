use nix::libc;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};
use crate::models::Error;

extern "C" fn sigint_handler(_sig: libc::c_int) {}

pub(super) fn install_signal_handlers() -> Result<(), Error> {

    // It is important that we un-set the SA_RESTART flag when installing the sigint handler, as
    // otherwise, we are prevented from breaking out of the even-reader-loop after receiving a
    // SIGINT signal.
    let sigint_action = SigAction::new(
        SigHandler::Handler(sigint_handler), SaFlags::empty(), SigSet::empty()
    );

    unsafe {
        sigaction(Signal::SIGINT, &sigint_action).map_err(|e| {
            Error::new(format!("Unable to install SIGINT handler: {e}"))
        })?;
    }

    Ok(())
}
