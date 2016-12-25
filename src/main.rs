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
extern crate sentry;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
extern crate fern;
extern crate time;
extern crate crypto;
extern crate regex;

use storage::FileStorage;
use config::Config;
use std::path::Path;
use std::str::FromStr;
use docopt::Docopt;
use sentry::{SentryCredential, Sentry};
use regex::Regex;

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

pub type BeamId = usize;
pub type Mtime = u64;

fn run(config: Config) {
    println!("Loaded configuration: {:?}", config);
    let storage = match FileStorage::open(&config.storage_path) {
        Ok(s) => s,
        Err(why) => panic!("Cannot open storage: {}", why)
    };

    let sentry = {
        let regex = Regex::new(
            r"^(?P<protocol>.*?)://(?P<key>.*?):(?P<secret>.*?)@(?P<host>.*?)/(?P<path>.*/)?(?P<project_id>.*)$").unwrap();

        regex.captures(&config.sentry_dsn)
            .map(|captures|
                 SentryCredential {
                     key: From::from(captures.name("key").unwrap()),
                     secret: From::from(captures.name("secret").unwrap()),
                     host: From::from(captures.name("host").unwrap()),
                     project_id: From::from(captures.name("project_id").unwrap()),
                 })
            .map(|credentials|
                 Sentry::new("Transporter".to_string(), VERSION.to_string(), "".to_string(), credentials)
            )
    };

    match server::listen(config, storage, sentry) {
        Err(why) => panic!("Server crashed: {}", why),
        _ => ()
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("Version {}", VERSION);
        return;
    }

    let config = Config::load(&Path::new(&args.arg_config)).unwrap();

    let log_level = log::LogLevelFilter::from_str(&config.log_level).unwrap();
    let mut output = vec![];
    output.push(fern::OutputConfig::stdout());

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, _: &log::LogLevel, location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            format!("[{}] {}", location.module_path(), msg)
        }),
        output: output,
        level: log_level,
    };

    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace).unwrap();
    run(config);
}
