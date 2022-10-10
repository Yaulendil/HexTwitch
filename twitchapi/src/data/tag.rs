use std::collections::HashMap;
use super::*;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub tag_id: String,
    pub is_auto: bool,
    pub localization_names: HashMap<String, String>,
    // pub localization_descriptions: HashMap<String, String>,
}

impl Endpoint for Tag {
    const PATH: &'static [&'static str] = &["tags", "streams"];
    // const PARAM: &'static str = "tag_id";
}
