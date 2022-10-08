// use std::sync::Arc;
// use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use parking_lot::{lock_api::RawRwLock, RwLock};
use twitchapi::*;
use crate::prefs::{HexPrefGet, HexPrefSet, HexPrefUnset, PREF_API_OAUTH2};


pub static API: RwLock<ApiHandler> = RwLock::const_new(
    RawRwLock::INIT,
    ApiHandler::new(),
);

const CLIENT_ID: &str = "";


#[derive(Debug)]
struct ImplicitAuthWait {
    url: Url,
    handle: JoinHandle<Option<AccessToken>>,
}

impl ImplicitAuthWait {
    fn is_done(&self) -> bool {
        self.handle.is_finished()
    }

    fn join(self) -> std::thread::Result<Option<AccessToken>> {
        self.handle.join()
    }

    fn spawn() -> Self {
        let client = client(ClientId::new(CLIENT_ID.into()));
        let (url, csrf) = implicit::auth_pre(&client);

        let handle = std::thread::spawn(move || implicit::authorize(csrf));

        Self { url, handle }
    }
}


#[derive(Debug)]
enum ApiState {
    Offline,
    Waiting(ImplicitAuthWait),
    Active(Api),
}

impl Default for ApiState {
    fn default() -> Self { Self::Offline }
}


#[derive(Debug)]
pub struct ApiHandler {
    state: ApiState,
}

impl ApiHandler {
    pub const fn new() -> Self {
        Self { state: ApiState::Offline }
    }

    pub const fn api(&self) -> Option<&Api> {
        match &self.state {
            ApiState::Active(api) => Some(api),
            _ => None,
        }
    }

    pub fn api_mut(&mut self) -> Option<&mut Api> {
        match &mut self.state {
            ApiState::Active(api) => Some(api),
            _ => None,
        }
    }

    pub const fn url(&self) -> Option<&Url> {
        match &self.state {
            ApiState::Waiting(wait) => Some(&wait.url),
            _ => None,
        }
    }

    /*pub fn client_id(&self) -> ClientId {
        ClientId::new(String::from(CLIENT_ID))
    }*/

    pub const fn is_waiting(&self) -> bool {
        matches!(self.state, ApiState::Waiting(_))
    }

    pub const fn is_valid(&self) -> bool {
        matches!(self.state, ApiState::Active(_))
    }

    pub fn clear(&mut self) {
        PREF_API_OAUTH2.unset().ok();
        self.state = ApiState::Offline;
    }

    pub fn stop(&mut self) {
        self.state = ApiState::Offline;
    }

    fn set_token(&mut self, token: AccessToken) -> Result<(), AccessToken> {
        let valid = token.validate(CLIENT_ID)?;
        let client = client(ClientId::new(CLIENT_ID.into()));
        self.state = ApiState::Active(Api::new(client, valid));

        Ok(())
    }

    pub fn validate(&mut self) -> bool {
        let storage = &PREF_API_OAUTH2;

        match &self.state {
            ApiState::Offline => {
                let valid: bool = match storage.get() {
                    Some(s) => match self.set_token(AccessToken::new(s)) {
                        Ok(()) => true,
                        Err(_) => false,
                    }
                    None => false,
                };

                if !valid {
                    let auth = ImplicitAuthWait::spawn();
                    self.state = ApiState::Waiting(auth);
                }

                valid
            }
            /*ApiState::Waiting(auth) => if auth.is_done() {
                let result = auth.join();
                self.state = ApiState::Offline;*/
            ApiState::Waiting(auth) => if auth.is_done() {
                let result = match std::mem::take(&mut self.state) {
                    ApiState::Waiting(auth) => auth.join(),
                    _ => unreachable!("state is known to be waiting"),
                };

                match result {
                    Ok(Some(token)) => {
                        let _stored = storage.set(token.secret()).is_ok();

                        match self.set_token(token) {
                            Ok(()) => true,
                            Err(_) => false,
                        }
                    }
                    Ok(None) => false,
                    Err(_) => false,
                }
            } else {
                false
            }
            ApiState::Active(_) => true,
        }
    }
}
