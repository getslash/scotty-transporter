mod beam;
mod storage;
mod config;
mod server;
mod error;
mod scotty;

extern crate rustc_serialize;
extern crate hyper;
extern crate docopt;
extern crate byteorder;
extern crate url;
extern crate raven;
#[macro_use] extern crate log;
extern crate env_logger;

use storage::FileStorage;
use config::Config;
use std::env::args;
use std::path::Path;
use docopt::Docopt;

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_config: String,
    flag_version: bool,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
static USAGE: &'static str = "
Usage:
    transporter <config>
    transporter --version

Options:
    --version   Print the version.
";

type BeamId = usize;

fn run(config: &Config) {
    println!("Loaded configuration: {:?}", config);
    let storage = match FileStorage::open(&config.storage_path) {
        Ok(s) => s,
        Err(why) => panic!("Cannot open storage: {}", why)
    };

    let raven = raven::Client::from_string(&config.sentry_dsn).unwrap();

    match server::listen(config, &storage, &raven) {
        Err(why) => panic!("Server crashed: {}", why),
        _ => ()
    }
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("Version {}", VERSION);
        return;
    }

    let config = match Config::load(&Path::new(&args.arg_config)) {
        Ok(c) => c,
        Err(why) => panic!("{}", why)
    };
    run(&config);
}
