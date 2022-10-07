pub extern crate oauth2;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

// pub mod data;
pub mod implicit;
mod token;
mod url;

pub use oauth2::{AccessToken, ClientId, CsrfToken};
pub use token::{TokenOps, TokenValid};
pub use url::Url;

use std::collections::{HashMap, HashSet};
use oauth2::{
    basic::{BasicClient, BasicErrorResponse},
    http::Method,
    RequestTokenError,
};
// use data::{ChannelData, Endpoint, Streams, TagData, Tags, Users};
use implicit::authorize;
use url::{url_auth, url_token};


const MAX_PARAMS: usize = 100;


mod http {
    //! Micro module to act as a namespace for HTTP functionality. Exists mainly
    //!     to make it less of a pain to switch backends, if ever needed.

    pub type Error = oauth2::ureq::Error;
    pub type Request = oauth2::HttpRequest;
    pub type Response = oauth2::HttpResponse;
    pub type Result = std::result::Result<Response, Error>;

    pub fn send(request: Request) -> Result {
        oauth2::ureq::http_client(request)
    }
}


#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("Failed to retrieve data: {0}")]
    Request(#[from] http::Error),
    #[error("Failed to validate: {0}")]
    Validate(http::Error),
    #[error("Failed to acquire API Access Token: {0}")]
    Token(#[from] RequestTokenError<http::Error, BasicErrorResponse>),
}


pub fn client(id: ClientId) -> BasicClient {
    BasicClient::new(id, None, url_auth(), Some(url_token()))
}


#[derive(Debug)]
pub struct Api {
    client: BasicClient,
    token: TokenValid,
}

impl Api {
    pub const fn new(client: BasicClient, token: TokenValid) -> Self {
        Self { client, token }
    }

    /*pub fn query_tags(&mut self, tag_set: HashSet<&str>)
        -> Result<HashMap<String, TagData>, ApiError>
    {
        let tag_vec: Vec<&str> = tag_set.into_iter().collect();

        let len: usize = tag_vec.len();
        let mut tag_map = HashMap::with_capacity(len);
        // print_info(format!("Retrieving data for {len} tags..."));

        for chunk in tag_vec.chunks(MAX_PARAMS) {
            let tags: Tags = self.request("tag_id", chunk)?;

            for tag in tags.data {
                tag_map.insert(tag.tag_id.clone(), tag);
            }
        }

        debug_assert!(tag_map.len() <= len);
        Ok(tag_map)
    }

    pub fn query_plain(&mut self, channels: Vec<&str>)
        -> Result<Vec<ChannelData>, ApiError>
    {
        let len: usize = channels.len();
        let mut results = Vec::with_capacity(len);
        print_info(format!("Retrieving data for {len} channels..."));

        for chunk in channels.chunks(MAX_PARAMS) {
            let streams: Streams = self.request("user_login", chunk)?;
            let users: Users = self.request("login", chunk)?;

            results.extend(users.join(streams));
        }

        if cfg!(feature = "stream_tags") {
            let none = Vec::new();
            let tag_set = results.iter()
                .flat_map(|channel| match &channel.stream {
                    Some(stream) => &stream.tag_ids,
                    None => &none,
                })
                .map(|tag| tag.as_str())
                .collect();
            let tag_map = self.query_tags(tag_set)?;

            for data in &mut results {
                if let Some(stream) = &data.stream {
                    for tag_id in &stream.tag_ids {
                        if let Some(tag) = tag_map.get(tag_id) {
                            data.tags.push(tag.clone());
                        }
                    }
                }
            }
        }

        debug_assert!(results.len() <= len);
        Ok(results)
    }

    pub fn query<'f>(&mut self, follow: FollowList<'f>)
        -> Result<FollowData<'f>, ApiError>
    {
        let channels: Vec<&str> = prepare_query(&follow);
        let data_vec: Vec<ChannelData> = self.query_plain(channels)?;
        let mut data_map: HashMap<String, ChannelData> = data_vec.into_iter()
            .map(|data| (data.user.login.clone(), data))
            .collect();

        let mut take_channels = |channels: Vec<FollowedChannel<'f>>| {
            channels.into_iter().map(|channel| ChannelResult::<'f> {
                data: data_map.remove(&channel.name.to_ascii_lowercase()),
                channel,
            }).collect::<Vec<ChannelResult<'f>>>()
        };

        let ungrouped = take_channels(follow.ungrouped);
        let groups = follow.groups.into_iter().map(|group| GroupResult {
            name: group.name,
            show: group.show,
            channels: take_channels(group.channels),
        }).collect();
        let temp = take_channels(follow.temp);

        debug_assert!(data_map.is_empty());
        Ok(FollowData { ungrouped, groups, temp })
    }*/
}

impl Api {
    /*/// Construct an HTTP [`Request`] from [`URL`], [`Method`], parameter, and a
    ///     sequence of values for the parameter.
    ///
    /// [`Request`]: http::Request
    /// [`URL`]: Url
    fn req_new<V: AsRef<str>>(
        &self,
        mut url: Url,
        method: Method,
        param: impl AsRef<str>,
        values: impl IntoIterator<Item=V>,
    ) -> http::Request {
        {
            let mut pairs = url.query_pairs_mut();
            // pairs.clear();
            pairs.extend_pairs(values.into_iter().map(|v| (&param, v)));
            // pairs.finish();
        }

        http::Request {
            url,
            method,
            headers: self.token.headers(self.client.client_id()),
            body: Vec::new(),
        }
    }

    /// Construct an HTTP `GET` [`Request`] from API endpoint, parameter, and a
    ///     sequence of values for the parameter. Then, send the request, wait
    ///     for a [`Response`], and attempt to deserialize the data.
    ///
    /// Returns [`ApiError`] if either [`req_get`] or [`from_slice`] returns an
    ///     error.
    ///
    /// [`Request`]: http::Request
    /// [`Response`]: http::Response
    ///
    /// [`req_get`]: Self::req_get
    /// [`from_slice`]: serde_json::from_slice
    fn request<T>(&self, param: &str, values: &[&str]) -> Result<T, ApiError>
        where T: Endpoint,
    {
        let req = self.req_new(T::url(), Method::GET, param, values);

        if cfg!(debug_assertions) {
            let res = http::send(req.clone())?;

            match serde_json::from_slice(&res.body) {
                Ok(val) => Ok(val),
                Err(err) => {
                    eprintln!(
                        "ERROR: {err}\
                        \nREQUEST: {method} {url}\
                        \nRESPONSE: {response}",
                        err = err,
                        method = req.method,
                        url = req.url,
                        // request = String::from_utf8_lossy(&req.body),
                        response = String::from_utf8_lossy(&res.body),
                    );
                    panic!("Failed to deserialize API response.");
                }
            }
        } else {
            let res = http::send(req)?;
            let out = serde_json::from_slice(&res.body)?;

            Ok(out)
        }
    }*/
}
