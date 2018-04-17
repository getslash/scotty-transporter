mod beam;
mod storage;
mod config;
mod server;
mod error;
mod scotty;

extern crate byteorder;
extern crate clap;
extern crate crypto;
extern crate fern;
#[macro_use]
extern crate log;
extern crate openssl_probe;
#[macro_use]
extern crate quick_error;
extern crate reqwest;
extern crate rustc_serialize;
extern crate sentry;
extern crate time;
extern crate url;

use storage::FileStorage;
use config::Config;
use std::path::Path;
use std::str::FromStr;
use clap::{App, Arg};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub type BeamId = usize;
pub type Mtime = u64;

fn run(config: Config) {
    println!("Loaded configuration: {:?}", config);
    let storage = match FileStorage::open(&config.storage_path) {
        Ok(s) => s,
        Err(why) => panic!("Cannot open storage: {}", why),
    };

    let _guard = sentry::init(config.sentry_dsn.clone());

    match server::listen(config, storage) {
        Err(why) => panic!("Server crashed: {}", why),
        _ => (),
    }
}

fn main() {
    let matches = App::new("Transporter")
        .version(VERSION)
        .about("Scotty's transporter server")
        .arg(
            Arg::with_name("config")
                .help("Path to the configuration file")
                .required(true)
                .index(1),
        )
        .get_matches();

    let config = Config::load(&Path::new(&matches.value_of("config").unwrap())).unwrap();

    openssl_probe::init_ssl_cert_env_vars();

    let log_level = log::LogLevelFilter::from_str(&config.log_level).unwrap();
    let mut output = vec![];
    output.push(fern::OutputConfig::stdout());

    let logger_config = fern::DispatchConfig {
        format: Box::new(
            |msg: &str, _: &log::LogLevel, location: &log::LogLocation| {
                // This is a fairly simple format, though it's possible to do more complicated ones.
                // This closure can contain any code, as long as it produces a String message.
                format!("[{}] {}", location.module_path(), msg)
            },
        ),
        output: output,
        level: log_level,
    };

    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace).unwrap();
    run(config);
}
