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

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                record.module_path().unwrap_or(""),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();;

    run(config);
}
