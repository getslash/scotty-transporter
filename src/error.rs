use std::error::Error;
use std::convert::From;
use byteorder::Error as ByteError;
use std::fmt;
use std::io::Error as IoError;
use super::scotty::ScottyError;

#[derive(Debug)]
pub enum TransporterError {
    InvalidClientMessageCode(u8),
    ByteError(ByteError),
    IoError(IoError),
    ScottyError(ScottyError),
    ClientEOF,
}

impl From<ByteError> for TransporterError {
    fn from(err: ByteError) -> TransporterError { TransporterError::ByteError(err) }
}

impl From<IoError> for TransporterError {
    fn from(err: IoError) -> TransporterError { TransporterError::IoError(err) }
}

impl From<ScottyError> for TransporterError {
    fn from(err: ScottyError) -> TransporterError { TransporterError::ScottyError(err) }
}

impl Error for TransporterError {
    fn description(&self) -> &str {
        return "";
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for TransporterError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TransporterError::InvalidClientMessageCode(code) => formatter.write_fmt(format_args!("Invalid message code: {}", code)),
            TransporterError::ByteError(ref error) => formatter.write_fmt(format_args!("Byte error: {}", error)),
            TransporterError::IoError(ref error) => formatter.write_fmt(format_args!("IO error: {}", error)),
            TransporterError::ScottyError(ref error) => formatter.write_fmt(format_args!("Scotty error: {}", error)),
            TransporterError::ClientEOF => formatter.write_str("Client close the connection in a middle of a beam"),
        }
    }
}

pub type TransporterResult<T> = Result<T, TransporterError>;
