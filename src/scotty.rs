use hyper;
use hyper::Client;
use hyper::header::ContentType;
use hyper::error::Error as HttpError;
use hyper::status::StatusCode;
use hyper::method::Method;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use rustc_serialize::json::{EncoderError, DecoderError, encode, decode};
use super::{BeamId, Mtime};
use std::io::Read;
use std::io::Error as IoError;

const TIME_TO_SLEEP : u64 = 5;
const MAX_ATTEMPTS : u64 = 60000 / TIME_TO_SLEEP * 1;

pub struct Scotty {
    url: String,
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
}

#[derive(RustcEncodable)]
struct FileUpdateRequest {
    success: bool,
    error: String,
    size: Option<usize>,
    mtime: Option<Mtime>,
    checksum: Option<String>
}

#[derive(RustcEncodable)]
struct BeamUpdateDocument<'a> {
    completed: bool,
    error: Option<&'a str>
}

#[derive(RustcEncodable)]
struct BeamUpdateRequest<'a> {
    beam: BeamUpdateDocument<'a>,
}

impl<'a> BeamUpdateRequest<'a> {
    fn new(completed: bool, error: Option<&'a str>) -> BeamUpdateRequest<'a> {
        BeamUpdateRequest{ beam: BeamUpdateDocument { completed: completed, error: error }}
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum ScottyError {
        EncoderError(err: EncoderError) {
            from()
            display("Encoder Error: {}", err)
        }
        DecoderError(err: DecoderError) {
            from()
            display("Decoder Error: {}", err)
        }
        HttpError(err: HttpError) {
            from()
            display("HTTP Error: {}", err)
        }
        ScottyError(code: StatusCode, url: String) {
            display("Scotty returned {} for {}", code, url)
        }
        ScottyIsDown {}
        IoError(err: IoError) {
            from()
            display("IO Error: {}", err)
        }
    }
}

type ScottyResult<T> = Result<T, ScottyError>;

impl Scotty {
    pub fn new(url: &String) -> Scotty{
        Scotty {
            url: url.clone(),
            json_mime: "application/json".parse().unwrap() }
    }

    fn send_request(&self, method: Method, url: String, json: &str) -> ScottyResult<String> {
        let client = Client::new();
        let mut content = String::new();
        for attempt in 0..MAX_ATTEMPTS {
            let mut response = {
                let request = client.request(method.clone(), &url[..])
                    .body(json)
                    .header(ContentType(self.json_mime.clone()));
                try!(request.send())
            };

            match response.status {
                StatusCode::Ok => {
                    try!(response.read_to_string(&mut content));
                    return Ok(content);
                },
                StatusCode::BadGateway | StatusCode::GatewayTimeout => {
                    error!("Scotty returned {}. Attempt {} out of {}", response.status, attempt + 1, MAX_ATTEMPTS);
                    sleep(Duration::from_secs(TIME_TO_SLEEP));
                },
                _ => { return Err(ScottyError::ScottyError(response.status, url)); }
            }
        }
        Err(ScottyError::ScottyIsDown)
    }

    pub fn file_beam_start(&mut self, beam_id: BeamId, file_name: &str) -> ScottyResult<(String, String, bool)> {
        let params = FilePostRequest { file_name: file_name.to_string(), beam_id: beam_id };
        let encoded_params = try!(encode::<FilePostRequest>(&params));
        let result = try!(
            self.send_request(Method::Post, format!("{}/files", self.url), &encoded_params));
        let file_params = try!(decode::<FilePostResponse>(&result));
        Ok((file_params.file_id, file_params.storage_name, file_params.should_beam))
    }

    pub fn file_beam_end(&mut self, file_id: &str, err: Option<&Error>, file_size: Option<usize>, file_checksum: Option<String>, mtime: Option<Mtime>) -> ScottyResult<()> {
        let error_string = match err {
            Some(err) => err.description(),
            _ => ""
        };
        let params = FileUpdateRequest { success: err.is_none(), error: error_string.to_string(), size: file_size, checksum: file_checksum, mtime: mtime};
        let encoded_params = try!(encode::<FileUpdateRequest>(&params));
        try!(
            self.send_request(
                Method::Put,
                format!("{}/files/{}", self.url, file_id),
                &encoded_params));
        Ok(())
    }

    pub fn complete_beam(&mut self, beam_id: BeamId, error: Option<&str>) -> ScottyResult<()> {
        let params = BeamUpdateRequest::new(true, error);
        let encoded_params = try!(encode::<BeamUpdateRequest>(&params));
        try!(
            self.send_request(Method::Put, format!("{}/beams/{}", self.url, beam_id), &encoded_params));
        Ok(())
    }
}
