use std::net::{TcpListener};
use std::thread;
use std::sync::Arc;
use std::io::Result;
use sentry::{Sentry,Event};
use super::beam::beam_up;
use super::config::Config;
use super::storage::FileStorage;

pub fn listen(config: Config, storage: FileStorage, sentry: Option<Sentry>) -> Result<()> {
    let listener = match TcpListener::bind(&config.bind_address[..]) {
        Ok(l) => l,
        Err(why) => panic!("Server bind error: {}", why)
    };

    let arc_sentry = sentry.map(|s| Arc::new(s));
    debug!("Debug messages are on");
    info!("Listening for connections in {}", config.bind_address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let storage = storage.clone();
                let config = config.clone();
                let raven = arc_sentry.clone();
                let mut tags = vec![];
                match stream.peer_addr() {
                    Err(why) => error!("Cannot get peer address for socket: {}", why),
                    Ok(peer) => tags.push((format!("peer"), format!("{}", peer))),
                };
                thread::spawn(move || {
                    match beam_up(stream, storage, config, &mut tags) {
                        Err(why) => {
                            if !why.is_disconnection() {
                                if let Some(raven) = raven {
                                    let mut event = Event::new(
                                        "Transporter", "error", &format!("{}", why),
                                        None, None, None, None, None, None);
                                    for tag in tags {
                                        let (key, value) = tag;
                                        event.push_tag(key, value);
                                    }
                                    raven.log_event(event);
                                }
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
