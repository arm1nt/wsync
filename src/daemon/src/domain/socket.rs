use std::env;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use crate::util::constants::SERVER_SOCKET_PATH_ENV_VAR;

#[derive(Debug)]
pub(crate) struct Error {
    pub(crate) msg: String
}

pub(crate) struct UnlinkingListener {
    path: PathBuf,
    pub listener: UnixListener
}

impl UnlinkingListener {

    pub fn bind() -> Result<Self, Error> {
        let path_env_var = env::var(SERVER_SOCKET_PATH_ENV_VAR).map_err(|_| {
            Error { msg: format!("{SERVER_SOCKET_PATH_ENV_VAR} env var is not set") }
        })?;

        let path: PathBuf = PathBuf::from(path_env_var);

        UnixListener::bind(&path)
            .map(|listener| { UnlinkingListener { path, listener }})
            .map_err(|e| { Error { msg: e.to_string() } })
    }

}

impl Drop for UnlinkingListener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
