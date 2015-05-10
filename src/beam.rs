use std::net::TcpStream;
use std::io::{Read, Write};
use std::cmp::min;
use super::storage::FileStorage;
use super::error::{TransporterResult, TransporterError};
use super::scotty::Scotty;
use super::config::Config;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};


const CHUNK_SIZE: usize = 4096usize;

#[derive(Debug)]
pub enum ClientMessages {
    BeamComplete,
    StartBeamingFile,
    FileChunk,
    FileDone
}

impl ClientMessages {
    fn from_u8(code: u8) -> TransporterResult<ClientMessages> {
        match code {
            0 => Ok(ClientMessages::BeamComplete),
            1 => Ok(ClientMessages::StartBeamingFile),
            2 => Ok(ClientMessages::FileChunk),
            3 => Ok(ClientMessages::FileDone),
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

fn download(stream: &mut TcpStream, storage: &FileStorage, file_id: &str) -> TransporterResult<usize> {
    let mut file = try!(storage.create(file_id));
    let mut length: usize = 0;
    let mut read_chunk = [0u8; CHUNK_SIZE];

    loop {
        let message_code = try!(ClientMessages::from_u8(try!(stream.read_u8())));
        match message_code {
            ClientMessages::FileChunk => (),
            ClientMessages::FileDone => return Ok(length),
            _ => return Err(TransporterError::UnexpectedClientMessageCode(message_code)),
        }

        let chunk_size = try!(stream.read_u32::<BigEndian>());
        let mut bytes_remaining = chunk_size as usize;
        while bytes_remaining > 0 {
            let to_read = min(bytes_remaining, read_chunk.len());
            let bytes_read = try!(stream.read(&mut read_chunk[0..to_read]));
            if bytes_read == 0 {
                return Err(TransporterError::ClientEOF);
            }
            try!(file.write_all(&mut read_chunk[0..bytes_read]));
            bytes_remaining -= bytes_read;
            length += bytes_read;
        }
    }
}

fn beam_file(beam_id: usize, stream: &mut TcpStream, storage: &FileStorage, scotty: &mut Scotty) -> TransporterResult<()> {
    let peer = try!(stream.peer_addr());
    let file_name = try!(read_file_name(stream));

    let (file_id, storage_name, should_beam) = try!(scotty.file_beam_start(beam_id, &file_name));

    if !should_beam {
        try!(stream.write_u8(ServerMessages::SkipFile as u8));
        return Ok(());
    }

    try!(stream.write_u8(ServerMessages::BeamFile as u8));

    info!("{} / {}: Beaming up {} to {}", peer, beam_id, file_name, storage_name);

    match download(stream, storage, &storage_name) {
        Ok(length) => {
            info!("Finished beaming up {} ({} bytes)", file_name, length);
            try!(scotty.file_beam_end(&file_id, None, Some(length)));
            try!(stream.write_u8(ServerMessages::FileBeamed as u8));
            Ok(())
            },
        Err(why) => {
            info!("Error beaming up {}: {}", file_name, why);
            try!(scotty.file_beam_end(&file_id, Some(&why), None));
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
            _ => return Err(TransporterError::UnexpectedClientMessageCode(message_code)),
        }
    }
}

pub fn beam_up(mut stream: TcpStream, storage: FileStorage, config: Config, error_tags: &mut Vec<(String, String)>) -> TransporterResult<()> {
    let beam_id = try!(stream.read_u64::<BigEndian>()) as usize;
    error_tags.push((format!("beam_id"), format!("{}", beam_id)));
    let mut scotty = Scotty::new(&config.scotty_url);
    info!("Received beam up request with beam id {}", beam_id);

    match beam_loop(beam_id, &mut stream, &storage, &mut scotty) {
        Ok(_) => {
            info!("Beam up completed");
            try!(scotty.complete_beam(beam_id, None));
            Ok(())
        },
        Err(why) => {
            error!("Beam up failed: {}", why);
            let error = format!("Transporter Error: {}", why);
            try!(scotty.complete_beam(beam_id, Some(&error)));
            Err(why)
        }
    }
}
