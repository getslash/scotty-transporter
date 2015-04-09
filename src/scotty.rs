use hyper;
use hyper::Client;
use hyper::net::HttpConnector;
use hyper::header::ContentType;
use hyper::error::HttpError;
use hyper::status::StatusCode;
use std::error::Error;
use std::fmt;
use rustc_serialize::json::{EncoderError, DecoderError, encode, decode};
use super::BeamId;
use std::io::Read;
use std::io::Error as IoError;

pub struct Scotty<'v> {
    url: String,
    client: Client<HttpConnector<'v>>,
    json_mime: hyper::mime::Mime
}

#[derive(RustcDecodable)]
struct FilePostResponse {
    file_id: String,
    storage_name: String,
    should_beam: bool
}

#[derive(RustcEncodable)]
struct FilePostRequest {
    file_name: String,
    beam_id: BeamId,
    file_size: usize
}

#[derive(RustcEncodable)]
struct FileUpdateRequest {
    success: bool,
    error: String
}

#[derive(RustcEncodable)]
struct BeamUpdateRequest {
    completed: bool,
}

macro_rules! check_response {
    ($response:ident, $url:ident) => (
        if $response.status != hyper::status::StatusCode::Ok {
            return Err(ScottyError::ScottyError($response.status, From::from(&$url as &str)));
        }
    )
}

#[derive(Debug)]
pub enum ScottyError {
    EncoderError(EncoderError),
    DecoderError(DecoderError),
    HttpError(HttpError),
    ScottyError(StatusCode, String),
    IoError(IoError),
}

impl Error for ScottyError {
    fn description(&self) -> &str { "Scotty Error" }
    fn cause(&self) -> Option<&Error> {
        match *self {
            ScottyError::EncoderError(ref e) => Some(e),
            ScottyError::DecoderError(ref e) => Some(e),
            ScottyError::HttpError(ref e) => Some(e),
            ScottyError::IoError(ref e) => Some(e),
            _ => None
        }
    }
}

impl fmt::Display for ScottyError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ScottyError::EncoderError(ref e) => formatter.write_fmt(format_args!("Encoder Error: {}", e)),
            ScottyError::DecoderError(ref e) => formatter.write_fmt(format_args!("Deocder Error: {}", e)),
            ScottyError::HttpError(ref e) => formatter.write_fmt(format_args!("Http Error: {}", e)),
            ScottyError::IoError(ref e) => formatter.write_fmt(format_args!("IO Error: {}", e)),
            ScottyError::ScottyError(code, ref url) => formatter.write_fmt(format_args!("Scotty returned {} for {}", code, url))
        }
    }
}

impl From<HttpError> for ScottyError {
    fn from(err: HttpError) -> ScottyError { ScottyError::HttpError(err) }
}

impl From<DecoderError> for ScottyError {
    fn from(err: DecoderError) -> ScottyError { ScottyError::DecoderError(err) }
}

impl From<EncoderError> for ScottyError {
    fn from(err: EncoderError) -> ScottyError { ScottyError::EncoderError(err) }
}

impl From<IoError> for ScottyError {
    fn from(err: IoError) -> ScottyError { ScottyError::IoError(err) }
}

type ScottyResult<T> = Result<T, ScottyError>;

impl<'v> Scotty<'v> {
    pub fn new(url: &String) -> Scotty<'v>{
        Scotty {
            url: url.clone(),
            client: Client::new(),
            json_mime: "application/json".parse().unwrap() }
    }

    pub fn file_beam_start(&mut self, beam_id: BeamId, file_name: &str, file_size: usize) -> ScottyResult<(String, String, bool)> {
        let url = format!("{}/files", self.url);
        let params = FilePostRequest { file_name: file_name.to_string(), beam_id: beam_id, file_size: file_size };
        let encoded_params = try!(encode::<FilePostRequest>(&params));
        let request = self.client.post(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()));
        let mut response = try!(request.send());
        check_response!(response, url);
        let mut content = String::new();
        try!(response.read_to_string(&mut content));
        let result = try!(decode::<FilePostResponse>(&content));
        Ok((result.file_id, result.storage_name, result.should_beam))
    }

    pub fn file_beam_end(&mut self, file_id: &str, err: Option<&Error>) -> ScottyResult<()> {
        let url = format!("{}/files/{}", self.url, file_id);
        let error_string = match err {
            Some(err) => err.description(),
            _ => ""
        };
        let params = FileUpdateRequest { success: err.is_none(), error: error_string.to_string() };
        let encoded_params = try!(encode::<FileUpdateRequest>(&params));
        let response = try!(self.client.put(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()))
            .send());
        check_response!(response, url);
        Ok(())
    }

    pub fn complete_beam(&mut self, beam_id: BeamId) -> ScottyResult<()> {
        let url = format!("{}/beams/{}", self.url, beam_id);
        let params = BeamUpdateRequest { completed: true };
        let encoded_params = try!(encode::<BeamUpdateRequest>(&params));
        let response = try!(self.client.put(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()))
            .send());
        check_response!(response, url);
        Ok(())
    }
}
