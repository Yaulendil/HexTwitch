use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use const_format::{formatcp, str_replace};
use oauth2::{AccessToken, basic::BasicClient, CsrfToken, RedirectUrl, Scope, url::Url};


const HOST: &str = "localhost";
const PORT: u16 = 8137;
const URL_REDIRECT: &str = formatcp!("http://{HOST}:{PORT}");


fn make_response(data: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\
        \r\nContent-Length: {size}\
        \r\n\
        \r\n{data}",
        size = data.len(),
        data = data,
    )
}


fn reply(stream: &mut TcpStream, data: &str) {
    let response = make_response(data);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}


fn reply_error(stream: &mut TcpStream, message: &str) {
    const PAGE_ERROR: &str = include_str!("html/error.html");
    reply(stream, &PAGE_ERROR.replace("!ERR!", message))
}


fn reply_final(stream: &mut TcpStream) {
    const PAGE_FINAL: &str = include_str!("html/final.html");
    reply(stream, PAGE_FINAL)
}


fn reply_redir(stream: &mut TcpStream) {
    const PAGE_REDIR: &str = str_replace!(
        include_str!("html/redir.html"),
        "URL_REDIRECT",
        URL_REDIRECT,
    );
    debug_assert!(PAGE_REDIR.contains(URL_REDIRECT));
    reply(stream, PAGE_REDIR)
}


fn request_read(stream: &mut TcpStream) -> Vec<u8> {
    const BUFFER: usize = 2048;

    let mut data: Vec<u8> = vec![0; BUFFER];
    let len: usize = stream.read(&mut data).unwrap();

    data.truncate(len);
    data
}


fn try_receive(
    stream: &mut TcpStream,
    csrf: &CsrfToken,
) -> Option<Result<String, String>> {
    let data = request_read(stream);
    let text = String::from_utf8_lossy(&data);

    let gotten = text.strip_prefix("GET ")?;
    let path_query = match gotten.split_once(' ') {
        Some((first, _)) => first,
        None => gotten,
    };

    let (_path, query) = path_query.split_once('?')?;
    let map: HashMap<&str, &str> = query.split('&').map(|kv| {
        match kv.split_once('=') {
            Some(k_v) => k_v,
            None => (kv, ""),
        }
    }).collect();

    if map.get("state")? == csrf.secret() {
        Some(Ok(String::from(*map.get("access_token")?)))
    } else {
        Some(Err(String::from("CSRF Token does not match.")))
    }
}


pub fn auth_pre(client: &BasicClient, scopes: &[&str]) -> (Url, CsrfToken) {
    let url_redirect = RedirectUrl::new(String::from(URL_REDIRECT))
            .expect("Invalid Redirect URL");

    client.authorize_url(CsrfToken::new_random)
        .add_scopes(scopes.iter().map(|s: &&str| -> Scope {
            Scope::new(String::from(*s))
        }))
        .set_redirect_uri(Cow::Owned(url_redirect))
        .use_implicit_flow()
        .url()
}


/// Prints a URL to authorize the application with Twitch, sets up a tiny HTTP
///     server, and blocks until receiving an Access Token via redirect.
pub fn authorize(csrf: CsrfToken) -> Option<AccessToken> {
    let listener = TcpListener::bind((HOST, PORT)).unwrap();

    for req in listener.incoming() {
        let stream = &mut req.unwrap();

        match try_receive(stream, &csrf) {
            Some(Ok(token)) => {
                reply_final(stream);
                return Some(AccessToken::new(token));
            }
            Some(Err(e)) => reply_error(stream, &e),
            None => reply_redir(stream),
        }
    }

    None
}
