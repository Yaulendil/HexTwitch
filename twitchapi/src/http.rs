//! Micro module to act as a namespace for HTTP functionality. Exists mainly to
//!     make it less of a pain to switch backends, if ever needed.

pub type Error = oauth2::ureq::Error;
pub type Request = oauth2::HttpRequest;
pub type Response = oauth2::HttpResponse;
pub type Result = std::result::Result<Response, Error>;


pub fn send(request: Request) -> Result {
    oauth2::ureq::http_client(request)
}
