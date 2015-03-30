use hyper;
use hyper::Client;
use hyper::net::HttpConnector;
use hyper::header::ContentType;
use std::error::Error;
use rustc_serialize::json;
use super::error::{TransporterResult, TransporterError};
use super::BeamId;
use std::io::Read;

pub struct Scotty<'v> {
    url: String,
    client: Client<HttpConnector<'v>>,
    json_mime: hyper::mime::Mime
}

#[derive(RustcDecodable)]
struct FilePostResponse {
    file_id: String,
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
            return Err(TransporterError::Message(format!("Scotty returned {} for {}", $response.status, $url)));
        }
    )
}

impl<'v> Scotty<'v> {
    pub fn new(url: &String) -> Scotty<'v>{
        Scotty {
            url: url.clone(),
            client: Client::new(),
            json_mime: "application/json".parse().unwrap() }
    }

    pub fn file_beam_start(&mut self, beam_id: BeamId, file_name: &str, file_size: usize) -> TransporterResult<(String, bool)> {
        let url = format!("{}/api/rest/files", self.url);
        let params = FilePostRequest { file_name: file_name.to_string(), beam_id: beam_id, file_size: file_size };
        debug!("Encoding params");
        let encoded_params = try!(json::encode::<FilePostRequest>(&params));
        debug!("Sending request to {} {}", url, encoded_params);
        let request = self.client.post(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()));
        let mut response = request.send().unwrap();
        debug!("checking response");
        check_response!(response, url);
        let mut content = String::new();
        debug!("reading response");
        try!(response.read_to_string(&mut content));
        debug!("decoding response");
        let result = try!(json::decode::<FilePostResponse>(&content));
        Ok((result.file_id, result.should_beam))
    }

    pub fn file_beam_end(&mut self, file_id: &str, err: Option<&Error>) -> TransporterResult<()> {
        let url = format!("{}/api/rest/file/{}", self.url, file_id);
        let error_string = match err {
            Some(err) => err.description(),
            _ => ""
        };
        let params = FileUpdateRequest { success: err.is_none(), error: error_string.to_string() };
        let encoded_params = try!(json::encode::<FileUpdateRequest>(&params));
        let response = try!(self.client.put(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()))
            .send());
        check_response!(response, url);
        Ok(())
    }

    pub fn complete_beam(&mut self, beam_id: BeamId) -> TransporterResult<()> {
        let url = format!("{}/api/rest/beam/{}", self.url, beam_id);
        let params = BeamUpdateRequest { completed: true };
        let encoded_params = try!(json::encode::<BeamUpdateRequest>(&params));
        let response = try!(self.client.put(&url[..])
            .body(&encoded_params[..])
            .header(ContentType(self.json_mime.clone()))
            .send());
        check_response!(response, url);
        Ok(())
    }
}