use chrono::{DateTime, Utc};
use super::*;


#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
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

impl Endpoint for Stream {
    const PATH: &'static [&'static str] = &["streams"];
    // const PARAM: &'static str = "user_login";
}
