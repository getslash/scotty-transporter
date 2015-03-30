use std::fs::File;
use std::path::Path;
use std::io::Read;
use rustc_serialize::json;
use super::error::TransporterResult;

#[derive(Debug, RustcDecodable, Clone)]
pub struct Config {
    pub storage_path: String,
    pub bind_address: String,
    pub scotty_url: String,
}

impl Config {
    pub fn load(path: &Path) -> TransporterResult<Config> {
        let mut raw_json = String::new();
        let _ = try!(File::open(path)).read_to_string(&mut raw_json);
        let json = try!(json::decode::<Config>(&raw_json));
        Ok(json)
    }
}