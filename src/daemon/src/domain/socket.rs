use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use wsync_config::{config, ConfigKey};

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
        let path = config()
            .get_path(ConfigKey::DaemonCommandSocketPath)
            .ok_or(Error { msg: "Config does not specify a path for a daemon command socket".to_string() })?;

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
