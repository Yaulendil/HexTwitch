use std::ops::Deref;
use oauth2::{AccessToken, http::{HeaderMap, HeaderValue, Method}};
use super::{http, url::url_validate};


#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct TokenValid(AccessToken);

impl TokenValid {
    #[allow(dead_code)]
    pub fn take(self) -> AccessToken { self.0 }
}

impl Deref for TokenValid {
    type Target = AccessToken;
    fn deref(&self) -> &Self::Target { &self.0 }
}


pub trait TokenOps: Sized {
    fn headers(&self, client_id: &str) -> HeaderMap;
    fn validate(self, client_id: &str) -> Result<TokenValid, Self>;
}

impl TokenOps for AccessToken {
    fn headers(&self, client_id: &str) -> HeaderMap {
        let header = format!("Bearer {}", self.secret());
        let bearer = HeaderValue::from_maybe_shared(header).unwrap();
        let client = HeaderValue::from_str(client_id).unwrap();

        let mut headers = HeaderMap::with_capacity(2);
        headers.insert("Authorization", bearer);
        headers.insert("Client-Id", client);
        headers
    }

    fn validate(self, client_id: &str) -> Result<TokenValid, Self> {
        let validation = http::Request {
            url: url_validate(),
            method: Method::GET,
            headers: self.headers(client_id),
            body: Vec::new(),
        };

        match http::send(validation) {
            Ok(_) => Ok(TokenValid(self)),
            Err(_) => Err(self),
        }
    }
}
