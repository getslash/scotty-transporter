use reqwest::{Client, Method, StatusCode};
use reqwest::header::ContentType;
use reqwest::Error as HttpError;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use rustc_serialize::json::{decode, encode, DecoderError, EncoderError};
use super::{BeamId, Mtime};
use std::io::Read;
use std::io::Error as IoError;

const TIME_TO_SLEEP: u64 = 5;
const MAX_ATTEMPTS: u64 = 60000 / TIME_TO_SLEEP * 1;

pub struct Scotty {
    url: String,
    client: Client,
}

#[derive(RustcDecodable)]
struct FilePostResponse {
    file_id: String,
    storage_name: String,
    should_beam: bool,
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
    checksum: Option<String>,
}

#[derive(RustcEncodable)]
struct BeamUpdateDocument<'a> {
    completed: bool,
    error: Option<&'a str>,
}

#[derive(RustcEncodable)]
struct BeamUpdateRequest<'a> {
    beam: BeamUpdateDocument<'a>,
}

impl<'a> BeamUpdateRequest<'a> {
    fn new(completed: bool, error: Option<&'a str>) -> BeamUpdateRequest<'a> {
        BeamUpdateRequest {
            beam: BeamUpdateDocument {
                completed: completed,
                error: error,
            },
        }
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

pub type ScottyResult<T> = Result<T, ScottyError>;

impl Scotty {
    pub fn new(url: &String) -> Scotty {
        Scotty {
            url: url.clone(),
            client: Client::new(),
        }
    }

    fn send_request(&self, method: Method, url: String, json: String) -> ScottyResult<String> {
        let mut content = String::new();
        for attempt in 0..MAX_ATTEMPTS {
            let mut response = self.client
                .request(method.clone(), &url)
                .body(json.clone())
                .header(ContentType::json())
                .send()?;

            let status_code = response.status();
            match status_code {
                StatusCode::Ok => {
                    response.read_to_string(&mut content)?;
                    return Ok(content);
                }
                StatusCode::BadGateway | StatusCode::GatewayTimeout => {
                    error!(
                        "Scotty returned {}. Attempt {} out of {}",
                        status_code,
                        attempt + 1,
                        MAX_ATTEMPTS
                    );
                    sleep(Duration::from_secs(TIME_TO_SLEEP));
                }
                _ => {
                    return Err(ScottyError::ScottyError(status_code, url));
                }
            }
        }
        Err(ScottyError::ScottyIsDown)
    }

    pub fn file_beam_start(
        &mut self,
        beam_id: BeamId,
        file_name: &str,
    ) -> ScottyResult<(String, String, bool)> {
        let params = FilePostRequest {
            file_name: file_name.to_string(),
            beam_id: beam_id,
        };
        let encoded_params = encode::<FilePostRequest>(&params)?;
        let result =
            self.send_request(Method::Post, format!("{}/files", self.url), encoded_params)?;
        let file_params = decode::<FilePostResponse>(&result)?;
        Ok((
            file_params.file_id,
            file_params.storage_name,
            file_params.should_beam,
        ))
    }

    pub fn file_beam_end(
        &mut self,
        file_id: &str,
        err: Option<&Error>,
        file_size: Option<usize>,
        file_checksum: Option<String>,
        mtime: Option<Mtime>,
    ) -> ScottyResult<()> {
        let error_string = match err {
            Some(err) => err.description(),
            _ => "",
        };
        let params = FileUpdateRequest {
            success: err.is_none(),
            error: error_string.to_string(),
            size: file_size,
            checksum: file_checksum,
            mtime: mtime,
        };
        self.send_request(
            Method::Put,
            format!("{}/files/{}", self.url, file_id),
            encode::<FileUpdateRequest>(&params)?,
        )?;
        Ok(())
    }

    pub fn complete_beam(&mut self, beam_id: BeamId, error: Option<&str>) -> ScottyResult<()> {
        let params = BeamUpdateRequest::new(true, error);
        self.send_request(
            Method::Put,
            format!("{}/beams/{}", self.url, beam_id),
            encode::<BeamUpdateRequest>(&params)?,
        )?;
        Ok(())
    }
}
