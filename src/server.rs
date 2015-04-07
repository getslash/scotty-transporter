use std::net::{TcpListener};
use std::thread;
use std::io::Result;
use super::beam::beam_up;
use super::config::Config;
use super::storage::FileStorage;


pub fn listen(config: &Config, storage: &FileStorage) -> Result<()> {
    let listener = match TcpListener::bind(&config.bind_address[..]) {
        Ok(l) => l,
        Err(why) => panic!("Server bind error: {}", why)
    };

    info!("Listening for connections in {}", config.bind_address);
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let storage = storage.clone();
                let config = config.clone();
                thread::spawn(move || {
                    match beam_up(&mut stream, &storage, &config) {
                        Err(why) => { error!("Connection closed: {:?}", why); },
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
