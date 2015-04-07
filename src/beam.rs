use std::net::TcpStream;
use std::io::{Read, Write};
use std::cmp::min;
use super::storage::FileStorage;
use super::error::{TransporterResult, TransporterError};
use super::scotty::Scotty;
use super::config::Config;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};


const CHUNK_SIZE: usize = 4096usize;

enum ClientMessages {
    BeamComplete,
    StartBeamingFile,
}



impl ClientMessages {
    fn from_u8(code: u8) -> TransporterResult<ClientMessages> {
        match code {
            0 => Ok(ClientMessages::BeamComplete),
            1 => Ok(ClientMessages::StartBeamingFile),
            _ => Err(TransporterError::InvalidClientMessageCode(code)),
        }
    }
}

enum ServerMessages {
    SkipFile = 0,
    BeamFile = 1,
    FileBeamed = 2,
}

fn read_file_name(stream: &mut TcpStream) -> TransporterResult<String> {
    let file_name_length = try!(stream.read_u16::<BigEndian>()) as usize;
    let mut file_name = String::new();
    let file_name_length_read = try!(stream.take(file_name_length as u64).read_to_string(&mut file_name));
    assert_eq!(file_name_length_read, file_name_length);
    Ok(file_name)
}

fn download(stream: &mut TcpStream, storage: &FileStorage, file_id: &str, length: usize) -> TransporterResult<()> {
    let mut file = try!(storage.create(file_id));
    let mut bytes_remaining = length;
    let mut read_chunk = [0u8; CHUNK_SIZE];

    while bytes_remaining > 0 {
        let to_read = min(bytes_remaining, read_chunk.len());
        let bytes_read = try!(stream.read(&mut read_chunk[0..to_read]));
        try!(file.write_all(&mut read_chunk[0..bytes_read]));
        bytes_remaining -= bytes_read;
    }

    Ok(())
}

fn beam_file(beam_id: usize, stream: &mut TcpStream, storage: &FileStorage, scotty: &mut Scotty) -> TransporterResult<()> {
    let peer = try!(stream.peer_addr());
    let file_length = try!(stream.read_u64::<BigEndian>()) as usize;
    let file_name = try!(read_file_name(stream));

    let (file_id, should_beam) = try!(scotty.file_beam_start(beam_id, &file_name, file_length));

    if !should_beam {
        try!(stream.write_u8(ServerMessages::SkipFile as u8));
        return Ok(());
    }

    try!(stream.write_u8(ServerMessages::BeamFile as u8));

    info!("{} / {}: Beaming up {} ({} bytes) to {}", peer, beam_id, file_name, file_length, file_id);

    match download(stream, storage, &file_id, file_length) {
        Ok(_) => {
            info!("Finished beaming up {}", file_name);
            try!(scotty.file_beam_end(&file_id, None));
            try!(stream.write_u8(ServerMessages::FileBeamed as u8));
            Ok(())
            },
        Err(why) => {
            info!("Error beaming up {}: {}", file_name, why);
            try!(scotty.file_beam_end(&file_id, Some(&why)));
            Err(why)
        }
    }
}

fn beam_loop(beam_id: usize, stream: &mut TcpStream, storage: &FileStorage, scotty: &mut Scotty) -> TransporterResult<()>
{
    loop {
        let message_code = try!(ClientMessages::from_u8(try!(stream.read_u8())));
        match message_code {
            ClientMessages::StartBeamingFile => try!(beam_file(beam_id, stream, storage, scotty)),
            ClientMessages::BeamComplete => return Ok(()),
        }
    }
}

pub fn beam_up(stream: &mut TcpStream, storage: &FileStorage, config: &Config) -> TransporterResult<()> {
    let beam_id = try!(stream.read_u64::<BigEndian>()) as usize;
    let mut scotty = Scotty::new(&config.scotty_url);
    info!("Received beam up request with beam id {}", beam_id);

    try!(beam_loop(beam_id, stream, storage, &mut scotty));
    info!("Beam up completed");
    try!(scotty.complete_beam(beam_id));
    Ok(())
}
