use std::fs::File;
use std::path::Path;
use std::io::Error as IoError;
use std::io::Read;
use serde_json;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub storage_path: String,
    pub bind_address: String,
    pub scotty_url: String,
    pub sentry_dsn: String,
    pub log_level: String,
}

quick_error! {
    #[derive(Debug)]
    pub enum ConfigError {
        IoError(err: IoError) {
            from()
            display("Configuration IO Error: {}", err)
        }
        DecodeError(err: serde_json::Error) {
            from()
            display("Configuration Decoding Error: {}", err)
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        let mut raw_json = String::new();
        let _ = File::open(path)
            .map_err(|e| ConfigError::IoError(e))?
            .read_to_string(&mut raw_json);
        let json: Config =
            serde_json::from_str(&raw_json).map_err(|e| ConfigError::DecodeError(e))?;
        Ok(json)
    }
}
