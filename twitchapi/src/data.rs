mod stream;
mod tag;
mod user;

pub use stream::*;
pub use tag::*;
pub use user::*;

use std::{collections::HashMap, ops::Deref};
use chrono::{DateTime, Utc};
use super::url::{Url, url_api_endpoint};


fn null_default<'de, D, T>(d: D) -> Result<T, D::Error> where
    D: serde::de::Deserializer<'de>,
    T: serde::de::Deserialize<'de> + Default,
{
    let opt: Option<T> = serde::de::Deserialize::deserialize(d)?;
    Ok(opt.unwrap_or_default())
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(transparent)]
pub struct ApiData<T> {
    pub data: Vec<T>,
}

impl<T> AsRef<[T]> for ApiData<T> {
    fn as_ref(&self) -> &[T] { self.data.as_slice() }
}

impl<T> Deref for ApiData<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target { &self.data }
}


/// Trait for data that can be retrieved from a specific Twitch API endpoint.
pub trait Endpoint {
    const PATH: &'static [&'static str];
    // const PARAM: &'static str;

    fn url() -> Url { url_api_endpoint(Self::PATH) }
}

/*impl<T: Endpoint> Endpoint for ApiData<T> {
    const PATH: &'static [&'static str] = T::PATH;
}*/
