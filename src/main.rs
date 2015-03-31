use storage::FileStorage;
use config::Config;
use std::env::args;
use std::path::Path;
use docopt::Docopt;

mod beam;
mod storage;
mod config;
mod server;
mod error;
mod scotty;

static USAGE: &'static str = "
Usage: transporter <config>
";

extern crate rustc_serialize;
extern crate hyper;
extern crate docopt;
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
    let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

    let config = match Config::load(&Path::new(&args.get_str("<config>"))) {
        Ok(c) => c,
        Err(why) => panic!("Cannot load configuration: {}", why)
    };
    run(&config);
}
