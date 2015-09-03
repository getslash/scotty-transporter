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
extern crate fern;
extern crate time;
extern crate crypto;

use storage::FileStorage;
use config::Config;
use std::env::args;
use std::path::Path;
use std::str::FromStr;
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
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("Version {}", VERSION);
        return;
    }

    let config = Config::load(&Path::new(&args.arg_config)).unwrap();

    let log_level = log::LogLevelFilter::from_str(&config.log_level).unwrap();
    let output = {
        let mut output = vec![];

        match config.log_file {
            Some(ref log_file) => output.push(fern::OutputConfig::file(log_file)),
            _ => ()
        };

        if config.log_stdout {
            output.push(fern::OutputConfig::stdout());
        }

        output
    };

    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, location: &log::LogLocation| {
            // This is a fairly simple format, though it's possible to do more complicated ones.
            // This closure can contain any code, as long as it produces a String message.
            format!("[{} {}/{}] {}", time::now().strftime("%Y-%m-%d %H:%M:%S").unwrap(), location.module_path(), level, msg)
        }),
        output: output,
        level: log_level,
    };

    fern::init_global_logger(logger_config, log::LogLevelFilter::Trace).unwrap();
    run(&config);
}
