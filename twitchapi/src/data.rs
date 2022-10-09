use std::collections::HashMap;
use chrono::{DateTime, Utc};
use super::url::{Url, url_api_endpoint};


fn null_default<'de, D, T>(d: D) -> Result<T, D::Error> where
    D: serde::de::Deserializer<'de>,
    T: serde::de::Deserialize<'de> + Default,
{
    let opt: Option<T> = serde::de::Deserialize::deserialize(d)?;
    Ok(opt.unwrap_or_default())
}


#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BroadcasterType {
    Normal,
    Affiliate,
    Partner,
    Unknown,
}


/// Trait for data that can be retrieved from a specific Twitch API endpoint.
pub trait Endpoint: serde::de::DeserializeOwned {
    const PATH: &'static [&'static str];
    // const PARAM: &'static str;

    fn url() -> Url { url_api_endpoint(Self::PATH) }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct StreamData {
    pub id: String,
    pub user_id: String,
    // pub user_login: String,
    // pub user_name: String,
    // pub game_id: String,
    pub game_name: String,
    // #[serde(rename = "type")]
    // pub stream_type: String,
    pub title: String,
    pub viewer_count: usize,
    pub started_at: DateTime<Utc>,

    #[serde(deserialize_with = "null_default")]
    pub tag_ids: Vec<String>,

    /*#[cfg(feature = "full_api")]
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,*/
}


#[derive(Debug, Deserialize)]
pub struct Streams {
    pub data: Vec<StreamData>,
}

impl Endpoint for Streams {
    const PATH: &'static [&'static str] = &["streams"];
    // const PARAM: &'static str = "user_login";
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagData {
    pub tag_id: String,
    pub is_auto: bool,
    pub localization_names: HashMap<String, String>,
    // pub localization_descriptions: HashMap<String, String>,
}


#[derive(Debug, Deserialize)]
pub struct Tags {
    pub data: Vec<TagData>,
}

impl Endpoint for Tags {
    const PATH: &'static [&'static str] = &["tags", "streams"];
    // const PARAM: &'static str = "tag_id";
}


#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub broadcaster_type: String,
    // #[serde(rename = "type")]
    // pub user_type: String,
    // pub view_count: usize,
    // pub created_at: DateTime<Utc>,

    /*#[cfg(feature = "full_api")]
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,*/
}

impl UserData {
    pub fn affiliation(&self) -> BroadcasterType {
        match self.broadcaster_type.as_str() {
            "" => BroadcasterType::Normal,
            "affiliate" => BroadcasterType::Affiliate,
            "partner" => BroadcasterType::Partner,
            _ => BroadcasterType::Unknown,
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct Users {
    pub data: Vec<UserData>,
}

impl Endpoint for Users {
    const PATH: &'static [&'static str] = &["users"];
    // const PARAM: &'static str = "login";
}


#[derive(Debug, Serialize)]
pub struct ChannelData {
    pub user: UserData,
    pub stream: Option<StreamData>,
    pub tags: Vec<TagData>,
}
