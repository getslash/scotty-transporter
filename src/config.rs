use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Error as IoError;
use std::io::Read;
use std::fmt;
use rustc_serialize::json;

#[derive(Debug, RustcDecodable, Clone)]
pub struct Config {
    pub storage_path: String,
    pub bind_address: String,
    pub scotty_url: String,
    pub sentry_dsn: String,
    pub log_level: String,
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(IoError),
    DecodeError(json::DecoderError)
}

impl Error for ConfigError {
    fn description(&self) -> &str { "Config Error" }
    fn cause(&self) -> Option<&Error> {
        match *self {
            ConfigError::IoError(ref e) => Some(e),
            ConfigError::DecodeError(ref e) => Some(e),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ConfigError::IoError(ref e) => formatter.write_fmt(format_args!("Configuration IO Error: {}", e)),
            ConfigError::DecodeError(ref e) => formatter.write_fmt(format_args!("Configuration Decoding Error: {}", e)),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        let mut raw_json = String::new();
        let _ = try!(File::open(path).map_err(|e| ConfigError::IoError(e)))
            .read_to_string(&mut raw_json);
        let json = try!(json::decode::<Config>(&raw_json)
            .map_err(|e| ConfigError::DecodeError(e)));
        Ok(json)
    }
}
