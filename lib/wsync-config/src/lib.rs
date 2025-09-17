use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;
use strum_macros::EnumString;
use crate::errors::Error;

pub mod errors;

pub(self) type Result<T> = std::result::Result<T, Error>;

#[derive(EnumString, Hash, Eq, PartialEq, Debug)]
pub enum ConfigKey {
    #[strum(serialize="WorkspaceConfigFilePath")]
    WorkspaceConfigFilePath,
    #[strum(serialize="DaemonCommandSocketPath")]
    DaemonCommandSocketPath,
    #[strum(serialize="MonitorExecutablePath")]
    MonitorExecutablePath,
    #[strum(serialize="LogDirectory")]
    LogDirectory,
}

#[derive(Debug)]
pub struct Config {
    pub(self) map: HashMap<ConfigKey, String>
}

impl Config {

    pub(self) fn new() -> Self {
        Config { map: HashMap::new() }
    }

    pub(self) fn insert(&mut self, key: ConfigKey, value: String) {
        self.map.insert(key, value);
    }

    pub fn get_string(&self, key: ConfigKey) -> Option<&String> {
        self.map.get(&key)
    }

    pub fn get_path(&self, key: ConfigKey) -> Option<PathBuf> {
        self.map
            .get(&key)
            .map(|val| PathBuf::from(val))
    }

}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get().expect("CONFIG not initialized")
}

pub(self) fn get_user_home_path() -> Result<PathBuf> {
    match env::var("HOME") {
        Ok(val) => Ok(PathBuf::from(val)),
        Err(e) => {
            Err(Error::Environment(format!( "Cannot get users home path: {e}")))
        }
    }
}

pub(self) fn validate_config_file(path: &PathBuf) -> Result<()> {

    if !path.exists() {
        return Err(Error::Environment(format!("'{:?}' does not exist!", path)));
    }

    if !path.is_file() {
        return Err(Error::Environment(format!("'{:?}' is not a file!", path)));
    }

    Ok(())
}

pub(self) fn get_wsync_config_path() -> Result<PathBuf> {
    let path;

    if let Ok(ws_config_file_path) = env::var("WSYNC_CONFIG_PATH") {
        path = PathBuf::from(ws_config_file_path);
    } else {
        let home_path = get_user_home_path()?;
        path = home_path.join(".wsync/wsync.config");
    }

    validate_config_file(&path)?;
    Ok(path)
}

pub(self) fn get_wsync_config_file() -> Result<File> {
    let path = get_wsync_config_path()?;

    File::open(path).map_err(|e| {
        Error::Io(format!("Unable to open wsync config file: {e}"))
    })
}

pub(self) fn validate_config_line_components(line: &String, components: &Option<(&str, &str)>) -> Result<()> {

    if components.is_none() {
        return Err(Error::MalformedConfigFile(
            format!("Config entry '{line}' does not conform to the format 'KEY=VALUE'")
        ));
    }

    let components = components.unwrap();

    if components.0.is_empty() {
        return Err(Error::MalformedConfigFile(
            format!("Config entry '{line}' has an empty key value!")
        ));
    }

    if components.1.is_empty() {
        return Err(Error::MalformedConfigFile(format!("Config entry '{line}' has no value!")));
    }

    Ok(())
}

pub(self) fn get_cfg_entry_components(cfg_line: String) -> Result<(String, String)> {
    let components: &Option<(&str, &str)> = &cfg_line.split_once("=");
    validate_config_line_components(&cfg_line, &components)?;
    let components = components.unwrap();

    Ok((components.0.to_string(), components.1.to_string()))
}

pub fn init_config() -> Result<()> {
    let mut config: Config = Config::new();
    let config_file = get_wsync_config_file()?;

    let reader = BufReader::new(config_file);
    for cfg_line in reader.lines() {
        let components = get_cfg_entry_components(cfg_line?)?;

        let key = ConfigKey::from_str(components.0.as_str()).map_err(|_e| {
            Error::MalformedConfigFile(format!("Invalid config key '{}' found!", components.0))
        })?;

        config.insert(key, components.1);
    }

    CONFIG.set(config).map_err(|set_cfg| {
        Error::Initialization(format!("Config already initialized: {set_cfg:?}"))
    })?;

    Ok(())
}
