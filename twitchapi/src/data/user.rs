use super::*;


#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BroadcasterType {
    Normal,
    Affiliate,
    Partner,
    Unknown,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct User {
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

impl User {
    pub fn affiliation(&self) -> BroadcasterType {
        match self.broadcaster_type.as_str() {
            "" => BroadcasterType::Normal,
            "affiliate" => BroadcasterType::Affiliate,
            "partner" => BroadcasterType::Partner,
            _ => BroadcasterType::Unknown,
        }
    }
}

impl Endpoint for User {
    const PATH: &'static [&'static str] = &["users"];
    // const PARAM: &'static str = "login";
}
