use std::env;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use log::info;
use crate::types::errors::SocketError;
use crate::util::constants::SERVER_SOCKET_PATH_EN_VAR;

pub(crate) struct UnlinkingListener {
    path: PathBuf,
    pub listener: UnixListener
}

impl UnlinkingListener {

    pub fn bind() -> Result<Self, SocketError> {
        let path_env_var = env::var(SERVER_SOCKET_PATH_EN_VAR).map_err(|e| {
            SocketError::new(format!("{SERVER_SOCKET_PATH_EN_VAR} env var is not set"))
        })?;

        let path: PathBuf = PathBuf::from(path_env_var);

        UnixListener::bind(&path)
            .map(|listener| { UnlinkingListener { path, listener }})
            .map_err(|e| {
                SocketError::new(e.to_string())
            })
    }

}

impl Drop for UnlinkingListener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
