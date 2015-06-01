use std::error::Error;
use std::convert::From;
use std::fmt;
use std::io::Error as IoError;
use super::scotty::ScottyError;
use super::beam::ClientMessages;

#[derive(Debug)]
pub enum TransporterError {
    InvalidClientMessageCode(u8),
    UnexpectedClientMessageCode(ClientMessages),
    ScottyError(ScottyError),
    ClientEOF,
    ClientIoError(IoError),
    StorageIoError(IoError),
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
            TransporterError::ScottyError(ref error) => Some(error),
            _ => None
        }
    }
}

impl TransporterError {
    pub fn is_disconnection(&self) -> bool {
        match *self {
            TransporterError::ClientIoError(_) => true,
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
            TransporterError::ScottyError(ref error) => write!(f, "Scotty error: {}", error),
            TransporterError::ClientEOF => write!(f, "Client close the connection in a middle of a beam"),
            TransporterError::ClientIoError(ref error) => write!(f, "Client IO error: {}", error),
            TransporterError::StorageIoError(ref error) => write!(f, "Storage IO error: {}", error),
        }
    }
}

pub type TransporterResult<T> = Result<T, TransporterError>;
