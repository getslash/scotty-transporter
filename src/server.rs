use std::net::{TcpListener};
use std::thread;
use std::io::Result;
use super::beam::beam_up;
use super::config::Config;
use super::storage::FileStorage;
use raven;


pub fn listen(config: &Config, storage: &FileStorage, raven: &raven::Client) -> Result<()> {
    let listener = match TcpListener::bind(&config.bind_address[..]) {
        Ok(l) => l,
        Err(why) => panic!("Server bind error: {}", why)
    };

    debug!("Debug messages are on");
    info!("Listening for connections in {}", config.bind_address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let storage = storage.clone();
                let config = config.clone();
                let raven = raven.clone();
                let mut error_tags: Vec<(String, String)> = Vec::new();
                match stream.peer_addr() {
                    Err(why) => error!("Cannot get peer address for socket: {}", why),
                    Ok(peer) => error_tags.push((format!("peer"), format!("{}", peer))),
                };
                thread::spawn(move || {
                    match beam_up(stream, storage, config, &mut error_tags) {
                        Err(why) => {
                            if !why.is_disconnection() {
                                let tags: Vec<_> = error_tags.iter().map(|&(ref a, ref b)| (a as &str, b as &str)).collect();
                                match raven.capture_error(&why, &tags) {
                                    Err(why) => error!("Cannot send error to Sentry: {}", why),
                                    _ => ()
                                };
                            }
                            error!("Connection closed: {}", why); },
                        Ok(_) => (),
                    };
                });
            },
            _ => (),
        }
    }

    drop(listener);
    Ok(())
}
