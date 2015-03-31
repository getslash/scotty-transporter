use std::error::{Error, FromError};
use byteorder::Error as ByteError;
use std::string::String;
use rustc_serialize::json;
use std::fmt;
use std::string::FromUtf8Error;
use std::io;
use hyper::HttpError;

#[derive(Debug)]
pub enum TransporterError {
    Chain(Box<Error + 'static>),
    Message(String)
}

macro_rules! from_error(
    ($err:ty) => {
        impl FromError<$err> for TransporterError {
            fn from_error(err: $err) -> Self {
                TransporterError::Chain(Box::new(err))
            }
        }
    }
);

from_error!(ByteError);
from_error!(json::DecoderError);
from_error!(json::EncoderError);
from_error!(FromUtf8Error);
from_error!(io::Error);

impl FromError<HttpError> for TransporterError {
    fn from_error(err: HttpError) -> Self {
        match err {
            HttpError::HttpIoError(err) => TransporterError::Chain(Box::new(err)),
            _ => TransporterError::Chain(Box::new(err))
        }
    }
}

impl Error for TransporterError {
    fn description(&self) -> &str {
        match *self {
            TransporterError::Chain(ref err) => err.description(),
            TransporterError::Message(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            TransporterError::Chain(ref err) => Some(&**err),
            _ => None,
        }
    }
}

impl fmt::Display for TransporterError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            TransporterError::Chain(ref err) => err.fmt(formatter),
            TransporterError::Message(ref s) => s.fmt(formatter)
        }
    }
}

pub fn error(s: String) -> TransporterError {
    TransporterError::Message(s)
}

pub type TransporterResult<T> = Result<T, TransporterError>;
