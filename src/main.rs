#![feature(core, slice_patterns)]

use storage::FileStorage;
use config::Config;
use std::env::args;
use std::path::Path;

mod beam;
mod storage;
mod config;
mod server;
mod error;
mod scotty;

extern crate rustc_serialize;
extern crate hyper;
extern crate byteorder;
extern crate url;
#[macro_use]extern crate log;
extern crate env_logger;

type BeamId = usize;

fn run(config: &Config) {
    println!("Loaded configuration: {:?}", config);
    let storage = match FileStorage::open(&config.storage_path) {
        Ok(s) => s,
        Err(why) => panic!("Cannot open storage: {}", why)
    };

    match server::listen(config, &storage) {
        Err(why) => panic!("Server crashed: {}", why),
        _ => ()
    }
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = args().collect();
    match args {
        [_, ref config_path] => {
            let config = match Config::load(&Path::new(config_path)) {
                Ok(c) => c,
                Err(why) => panic!("Cannot load configuration: {}", why)
            };
            run(&config);
        },
        _ => {
            println!("Usage: transporter [config file]");
            return;
        }
    }
}
