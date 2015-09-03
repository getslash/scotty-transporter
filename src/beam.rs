use std::net::TcpStream;
use std::io::{Read, Write};
use std::cmp::min;
use super::storage::FileStorage;
use super::error::{TransporterResult, TransporterError};
use super::scotty::Scotty;
use super::config::Config;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use byteorder::Error as ByteError;
use crypto::sha2::Sha512;
use crypto::digest::Digest;


const CHUNK_SIZE: usize = 1048576usize;

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

type FileData = (usize, Sha512);

fn map_byte_err(error: ByteError) -> TransporterError {
    match error {
        ByteError::UnexpectedEOF => TransporterError::ClientEOF,
        ByteError::Io(io) => TransporterError::ClientIoError(io)
    }
}

fn read_file_name(stream: &mut TcpStream) -> TransporterResult<String> {
    let file_name_length = try!(stream.read_u16::<BigEndian>().map_err(map_byte_err)) as usize;
    let mut file_name = String::new();
    let file_name_length_read = try!(stream.take(file_name_length as u64).read_to_string(&mut file_name).map_err(
        |io| TransporterError::ClientIoError(io)));
    assert_eq!(file_name_length_read, file_name_length);
    Ok(file_name)
}

fn download(stream: &mut TcpStream, storage: &FileStorage, file_id: &str) -> TransporterResult<FileData> {
    let mut file = try!(storage.create(file_id));
    let mut checksum = Sha512::new();
    let mut length: usize = 0;
    let mut read_chunk = [0u8; CHUNK_SIZE];

    loop {
        let message_code = try!(stream.read_u8().map_err(map_byte_err)
            .and_then(|m| ClientMessages::from_u8(m)));
        match message_code {
            ClientMessages::FileChunk => (),
            ClientMessages::FileDone => return Ok((length, checksum)),
            _ => return Err(TransporterError::UnexpectedClientMessageCode(message_code)),
        }

        let chunk_size = try!(stream.read_u32::<BigEndian>().map_err(map_byte_err));
        let mut bytes_remaining = chunk_size as usize;
        while bytes_remaining > 0 {
            let to_read = min(bytes_remaining, read_chunk.len());
            let bytes_read = try!(
                stream.read(&mut read_chunk[0..to_read]).map_err(|io| TransporterError::ClientIoError(io)));
            if bytes_read == 0 {
                return Err(TransporterError::ClientEOF);
            }
            checksum.input(&read_chunk[0..bytes_read]);
            try!(file.write_all(&mut read_chunk[0..bytes_read]).map_err(|io| TransporterError::StorageIoError(io)));
            bytes_remaining -= bytes_read;
            length += bytes_read;
        }
    }
}

fn beam_file(beam_id: usize, stream: &mut TcpStream, storage: &FileStorage, scotty: &mut Scotty) -> TransporterResult<()> {
    debug!("{}: Got a request to beam up file", beam_id);
    let file_name = try!(read_file_name(stream));
    debug!("{}: File name is {}", beam_id, file_name);

    let (file_id, storage_name, should_beam) = try!(scotty.file_beam_start(beam_id, &file_name));
    debug!("{}: ID of {} is {}.", beam_id, file_name, file_id);

    if !should_beam {
        debug!("{}: Notifying the client that we should'nt beam {}.", beam_id, file_id);
        try!(stream.write_u8(ServerMessages::SkipFile as u8).map_err(map_byte_err));
        return Ok(());
    }

    debug!("{}: Notifying the client that we should beam {}.", beam_id, file_id);
    try!(stream.write_u8(ServerMessages::BeamFile as u8).map_err(map_byte_err));

    info!("{}: Beaming up {} to {}", beam_id, file_name, storage_name);

    match download(stream, storage, &storage_name) {
        Ok(data) => {
            let (length, mut checksum) = data;
            info!("Finished beaming up {} ({} bytes)", file_name, length);
            try!(scotty.file_beam_end(&file_id, None, Some(length), Some(checksum.result_str())));
            try!(stream.write_u8(ServerMessages::FileBeamed as u8).map_err(map_byte_err));
            Ok(())
            },
        Err(why) => {
            info!("Error beaming up {}: {}", file_name, why);
            try!(scotty.file_beam_end(&file_id, Some(&why), None, None));
            Err(why)
        }
    }
}

fn beam_loop(beam_id: usize, stream: &mut TcpStream, storage: &FileStorage, scotty: &mut Scotty) -> TransporterResult<()>
{
    loop {
        let message_code = try!(ClientMessages::from_u8(try!(stream.read_u8().map_err(map_byte_err))));
        match message_code {
            ClientMessages::StartBeamingFile => try!(beam_file(beam_id, stream, storage, scotty)),
            ClientMessages::BeamComplete => return Ok(()),
            _ => return Err(TransporterError::UnexpectedClientMessageCode(message_code)),
        }
    }
}

pub fn beam_up(mut stream: TcpStream, storage: FileStorage, config: Config, error_tags: &mut Vec<(String, String)>) -> TransporterResult<()> {
    let beam_id = try!(stream.read_u64::<BigEndian>().map_err(map_byte_err)) as usize;
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
