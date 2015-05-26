use std::error::Error;
use std::convert::From;
use byteorder::Error as ByteError;
use std::fmt;
use std::io::Error as IoError;
use super::scotty::ScottyError;
use super::beam::ClientMessages;

#[derive(Debug)]
pub enum TransporterError {
    InvalidClientMessageCode(u8),
    UnexpectedClientMessageCode(ClientMessages),
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
        match *self {
            TransporterError::ByteError(ref error) => Some(error),
            TransporterError::IoError(ref error) => Some(error),
            TransporterError::ScottyError(ref error) => Some(error),
            _ => None
        }
    }
}

impl TransporterError {
    pub fn is_disconnection(&self) -> bool {
        match *self {
            TransporterError::ByteError(ref byte_error) => match *byte_error {
                ByteError::UnexpectedEOF => true,
                _ => false
            },
            TransporterError::ClientEOF => true,
            _ => false
        }
    }
}

impl fmt::Display for TransporterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TransporterError::InvalidClientMessageCode(code) => write!(f, "Invalid message code: {}", code),
            TransporterError::UnexpectedClientMessageCode(ref code) => write!(f, "Unexpected message code: {:?}", code),
            TransporterError::ByteError(ref error) => write!(f, "Byte error: {}", error),
            TransporterError::IoError(ref error) => write!(f, "IO error: {}", error),
            TransporterError::ScottyError(ref error) => write!(f, "Scotty error: {}", error),
            TransporterError::ClientEOF => write!(f, "Client close the connection in a middle of a beam"),
        }
    }
}

pub type TransporterResult<T> = Result<T, TransporterError>;
