pub use oauth2::{AuthUrl, TokenUrl, url::Url};


const URL_API: &str = "https://api.twitch.tv/helix";
const ERR_API: &str = "Invalid API URL";

const URL_AUTH: &str = "https://id.twitch.tv/oauth2/authorize";
const ERR_AUTH: &str = "Invalid Auth URL";

const URL_TOKEN: &str = "https://id.twitch.tv/oauth2/token";
const ERR_TOKEN: &str = "Invalid Token URL";

const URL_VALIDATE: &str = "https://id.twitch.tv/oauth2/validate";
const ERR_VALIDATE: &str = "Invalid Validation URL";


pub fn url_api() -> Url {
    Url::parse(URL_API).expect(ERR_API)
}

pub fn url_api_endpoint(path: &[&str]) -> Url {
    let mut url = url_api();
    url.path_segments_mut().expect(ERR_API).extend(path);
    url
}


pub fn url_auth() -> AuthUrl {
    AuthUrl::new(String::from(URL_AUTH)).expect(ERR_AUTH)
}


pub fn url_token() -> TokenUrl {
    TokenUrl::new(String::from(URL_TOKEN)).expect(ERR_TOKEN)
}


pub fn url_validate() -> Url {
    Url::parse(URL_VALIDATE).expect(ERR_VALIDATE)
}
