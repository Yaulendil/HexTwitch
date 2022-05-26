//! Module for the splitting of IRCv3 strings into a more usable format.

use std::{
    collections::HashMap,
    convert::Infallible,
    fmt,
};


/// Given a string which may contain characters which are not allowed in an IRC
///     Tag String, replace all such characters with escaped substitutions.
///
/// Allocates a new String.
///
/// Input: `&str`
/// Return: `String`
pub fn escape(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 20);

    for each in line.chars() {
        match each {
            '\\' => out.push_str(r"\\"),
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            ' ' => out.push_str(r"\s"),
            ';' => out.push_str(r"\:"),
            other => out.push(other),
        }
    }

    out
}


/// Given a string which has had characters in it replaced with sanitized
///     stand-ins, replace the stand-ins with the special characters they
///     represent.
///
/// Allocates a new String.
///
/// Input: `&str`
/// Return: `String`
pub fn unescape(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut iter = line.chars();

    while let Some(first) = iter.next() {
        if first == '\\' {
            match iter.next() {
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('s') => out.push(' '),
                Some(':') => out.push(';'),
                None => out.push(first),
                Some(other) => {
                    out.push(first);
                    out.push(other);
                }
            }
        } else { out.push(first) }
    }

    out
}


/// Create a new owned `String` from a `&str`, but only if it is not empty; If
///     it is empty, instead return `None`.
fn owned_not_empty(instr: &str) -> Option<String> {
    if instr.is_empty() {
        None
    } else {
        Some(String::from(instr))
    }
}


fn find_byte(line: &str, delim: u8) -> Option<usize> {
    for (i, b) in line.as_bytes().iter().enumerate() {
        if delim.eq(b) {
            return Some(i);
        }
    }

    None
}


/// Split a `&str` at the first occurrence of a delimiting `u8`.
pub fn split_at_byte(line: &str, delim: u8) -> (&str, &str) {
    match find_byte(line, delim) {
        Some(idx) => (&line[..idx], &line[idx + 1..]),
        None => (line, ""),
    }
}


/// Split a `&str` at the first occurrence of a delimiting `char`.
pub fn split_at_char(line: &str, delim: char) -> (&str, &str) {
    let bytes = delim.len_utf8();

    if bytes == 1 {
        match find_byte(line, delim as u8) {
            Some(idx) => (&line[..idx], &line[idx + 1..]),
            None => (line, ""),
        }
    } else {
        match line.find(delim) {
            Some(idx) => (&line[..idx], &line[idx + bytes..]),
            None => (line, ""),
        }
    }
}


/// Split a `&str` at the first occurrence of another delimiting `&str`.
fn split_at_str<'a>(line: &'a str, delim: &str) -> (&'a str, &'a str) {
    match line.find(delim) {
        Some(idx) => (&line[..idx], &line[idx + delim.len()..]),
        None => (line, ""),
    }
}


#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Signature(String);

impl Signature {
    pub fn new(
        channel: Option<impl AsRef<str>>,
        author: Result<impl AsRef<str>, ()>,
    ) -> Self {
        Self(format!(
            "{:?}:{:?}",
            channel.as_ref().map(|s| s.as_ref()),
            author.as_ref().map(|s| s.as_ref()),
        ))
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}


/// Prefix: A string used by Servers "to indicate the true origin of a message".
///     It may be either the hostname of a Server, or a string describing a User
///     and possibly its hostname.
#[derive(Debug, PartialEq)]
pub enum Prefix {
    ServerName(String),
    User {
        nick: String,
        user: Option<String>,
        host: Option<String>,
    },
}

impl Prefix {
    /// Name: Return a representation of the author of a Message, intended for a
    ///     human to read. If the Prefix is a Server Name, it will be that; If
    ///     it is instead a User String, it will be the Nick.
    ///
    /// Return: `&str`
    pub fn name(&self) -> &str {
        match self {
            Self::ServerName(server) => server,
            Self::User { nick, .. } => nick,
        }
    }

    pub fn server(&self) -> Option<&str> {
        match self {
            Self::ServerName(server) => Some(server),
            Self::User { host, .. } => host.as_ref().map(String::as_str),
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ServerName(server) => f.write_str(server),

            Self::User { nick, user: Some(user), host: Some(host) } =>
                write!(f, "{}!{}@{}", nick, user, host),

            Self::User { nick, user: Some(user), .. } =>
                write!(f, "{}!{}", nick, user),

            Self::User { nick, host: Some(host), .. } =>
                write!(f, "{}@{}", nick, host),

            Self::User { nick, .. } => f.write_str(nick),
        }
    }
}

impl std::str::FromStr for Prefix {
    type Err = Infallible;

    /// Split an IRC Prefix string into an Author.
    ///
    /// Input: `&str`
    /// Return: `Result<Prefix, !>`
    fn from_str(nick_user_host: &str) -> Result<Self, Self::Err> {
        if nick_user_host.contains('.') && !nick_user_host.contains('@') {
            Ok(Self::ServerName(String::from(nick_user_host)))
        } else {
            let (nick_user, hostname) = split_at_char(nick_user_host, '@');
            let (nickname, username) = split_at_char(nick_user, '!');

            Ok(Self::User {
                nick: String::from(nickname),
                user: owned_not_empty(username),
                host: owned_not_empty(hostname),
            })
        }
    }
}


/// An IRC Message in a usable structure.
#[derive(Debug, PartialEq)]
pub struct Message {
    /// UserString: `nick[!user][@host]`
    pub prefix: Prefix,
    /// IRC Command.
    pub command: String,
    /// Arguments passed to the Command.
    pub args: Vec<String>,
    /// Remainder of the Message.
    pub trail: String,
    /// IRCv3 Tags. Strings mapped to Strings. This will be `None` if the
    ///     original message did not include a Tags segment.
    pub tags: Option<HashMap<String, String>>,
}

impl Message {
    /// Author: Return the name, put simply, of the source of this Message.
    ///
    /// Return: `&str`
    pub fn author(&self) -> &str { self.prefix.name() }

    /// Get a `String` representing this `Message` which will identify it.
    ///
    /// Return: [`Signature`]
    pub fn get_signature(&self) -> Signature {
        Signature::new(self.args.first(), Ok(self.author()))
    }

    /// Check whether this `Message` includes IRC Tags.
    ///
    /// Return: `bool`
    #[inline]
    pub fn has_tags(&self) -> bool { self.tags.is_some() }

    /// Retrieve a Tag from the `Message`.
    ///
    /// Input: `&str`
    /// Return: `Option<String>`
    pub fn get_tag(&self, key: &str) -> Option<String> {
        //  NOTE: Benchmarking seems to show that it is faster to unescape Tag
        //      values on demand, here, rather than ahead of time.
        Some(unescape(self.tags.as_ref()?.get(key)?))
    }

    /// Set a Tag on the `Message`. If the Tag was already present, its old
    ///     value is returned. If the `Message` has `None` for its Tags field,
    ///     `Err(())` is returned.
    ///
    /// Input: `&str`, `&str`
    /// Return: `Result<Option<String>, ()>`
    pub fn set_tag(&mut self, key: &str, value: &str)
                   -> Result<Option<String>, ()>
    {
        self.tags.as_mut().ok_or(())
            .map(|tags| tags.insert(String::from(key), escape(value)))
    }
}

impl fmt::Display for Message {
    /// Format this `Message` into a format which is suitable for sending over
    ///     IRC.
    ///
    /// Return: `fmt::Result`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(tags) = &self.tags {
            let mut tagseq: Vec<String> = tags
                .iter()
                .map(|(key, val)|
                    if val.is_empty() {
                        key.to_owned()
                    } else {
                        format!("{}={}", key, val)
                    }
                )
                .collect();

            tagseq.sort_unstable();
            write!(f, "@{} ", tagseq.join(";"))?;
        }

        write!(f, ":{} {}", self.prefix, self.command)?;
        for arg in &self.args { write!(f, " {}", arg)?; }
        if !self.trail.is_empty() { write!(f, " :{}", self.trail)?; }

        Ok(())
    }
}

impl std::str::FromStr for Message {
    type Err = Infallible;

    /// Split a raw IRC string into a usable `Message`.
    ///
    /// Input: `&str`
    /// Return: `Result<Message, !>`
    fn from_str(full_str: &str) -> Result<Self, Self::Err> {
        //  "@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Break the line down.
        let (full_message, tags): (&str, Option<HashMap<String, String>>) = {
            if full_str.starts_with('@') {
                //  The Tags String is the first half of the original message
                //      received by IRC. The "regular" message begins after the
                //      first space.
                let (tag_str, main_str) = split_at_char(&full_str, ' ');

                (main_str, Some(
                    tag_str[1..]
                        .split(';')
                        .map(|kvp| {
                            let (key, val) = split_at_char(kvp, '=');

                            (key.to_owned(), val.to_owned())
                        })
                        .collect::<HashMap<String, String>>()
                ))
            } else {
                (full_str, None)
            }
        };
        //  { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        //  ":asdfqwert!asdfqwert@twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Now, parse the message itself.
        //  This format is specified in Section 2.3.1 of RFC 1459.
        let (prefix, message) = if full_message.starts_with(':') {
            //  This Message has a Prefix. The Prefix is most likely hostname
            //      and/or server info. It ends at the first space.
            split_at_char(&full_message[1..], ' ')
        } else {
            ("", full_message)
        };
        //  "asdfqwert!asdfqwert@twitch.tv"
        //  "PRIVMSG #zxcv arg2 :this is a message"

        //  The trailing data is found after a space and a colon. Everything up
        //      to that point is the IRC Command and any Arguments passed to it.
        let (cmd_and_args, trail) = split_at_str(message, " :");
        //  "PRIVMSG #zxcv arg2"
        //  "this is a message"

        //  The Command is the first word before any Arguments.
        let (command, args_str) = split_at_char(cmd_and_args, ' ');
        //  "PRIVMSG"
        //  "#zxcv arg2"

        //  The Arguments should be split apart into a Vector of `String`s.
        let args: Vec<String> = args_str.split_ascii_whitespace()
            .map(String::from)
            .collect();
        //  ["#zxcv", "arg2"]

        //  Compile everything into a Message Struct, and send it back up.
        Ok(Self {
            prefix: prefix.parse()?,  // "asdfqwert!asdfqwert@twitch.tv"
            command: String::from(command),  // "PRIVMSG"
            args,  // ["#zxcv", "arg2"]
            trail: String::from(trail),  // "this is a message"
            tags,  // { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        })
    }
}


#[cfg(test)]
pub mod tests_irc {
    use super::*;

    const MSG_WITHOUT_TAGS: &str = r":asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv WHISPER thyself :asdf";
    const SAMPLES: &[&str] = &[
        MSG_WITHOUT_TAGS,
        r"@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message",
        r"@badges=subscriber/24;badge-info=subscriber/25,bits/1;emote-sets=0,2652,15749,19194,230961,320370,1228598;user-type= :tmi.twitch.tv USERSTATE #asdfqwert",
        r"@login=somejerk;room-id=;target-msg-id=15604c60-4d3b-8c1c-8e7a-c9ec2fb6c0cf;tmi-sent-ts=-6745368778951 :tmi.twitch.tv CLEARMSG #coolchannel :get a real job noob",
        r"@room-id=;target-user-id=8675309;tmi-sent-ts=1582958744397 :tmi.twitch.tv CLEARCHAT #coolchannel :somejerk",
        r"@badges=;color=#DABEEF;display-name=Asdf\sQwert;emotes=;message-id=2;thread-id=1337-9001;turbo=0;user-id=123456789;user-type= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv WHISPER thyself :asdf"
    ];
    const TEST_KEY: &str = "asdf";
    const TEST_VAL_1: &str = "qwert";
    const TEST_VAL_2: &str = "z x : c v";

    /// Test to confirm that converting back and forth between Message and text
    ///     will always produce the same results.
    #[test]
    fn test_irc_consistency() {
        for init in SAMPLES {
            let to_irc: Message = init.parse().expect("Failed to parse initial string.");
            let from_irc: String = to_irc.to_string();
            let back_to_irc: Message = from_irc.parse().expect("Failed to re-parse second string.");
            let back_from_irc: String = back_to_irc.to_string();

            assert_eq!(
                to_irc,
                back_to_irc,
                "Message produces a String which in turn DOES NOT produce an identical Message.",
            );
            assert_eq!(
                from_irc,
                back_from_irc,
                "Strings from Messages are not consistent.",
            );
        }
    }

    /// Test to confirm that manipulation of tags is working, and is preserved
    ///     across conversions between Message and text.
    #[test]
    fn test_set_tag() {
        let mut tagless: Message = MSG_WITHOUT_TAGS.parse()
            .expect("Failed to parse tagless sample.");
        assert_eq!(
            None,
            tagless.get_tag(TEST_KEY),
            "'Tagless' Message returns value for test key.",
        );
        assert_eq!(
            Err(()),
            tagless.set_tag(TEST_KEY, TEST_VAL_1),
            "Insertion of Tag does not fail on Message without Tags.",
        );

        for init in SAMPLES {
            let mut to_irc: Message = init.parse()
                .expect("Failed to parse initial string.");

            if to_irc.has_tags() {
                assert_eq!(
                    None,
                    to_irc.get_tag(TEST_KEY),
                    "Initial Message already has the test key.",
                );
                assert_eq!(
                    None,
                    to_irc.set_tag(TEST_KEY, TEST_VAL_1)
                        .expect("Insertion of new key returns Err"),
                    "Insertion of new key does not return None.",
                );
                assert_eq!(
                    TEST_VAL_1,
                    to_irc.set_tag(TEST_KEY, TEST_VAL_2)
                        .expect("Insertion of extant key returns Err")
                        .expect("Insertion of extant key does not return a \
                            previous value.")
                        .as_str(),
                    "Insertion of extant key returns incorrect value.",
                );
                assert_eq!(
                    Some(String::from(TEST_VAL_2)),
                    to_irc.get_tag(TEST_KEY),
                    "Test key cannot be correctly retrieved after insertion.",
                );

                let from_irc: String = to_irc.to_string();
                let back_to_irc: Message = from_irc.parse()
                    .expect("Failed to re-parse second string.");
                let back_from_irc: String = back_to_irc.to_string();

                assert_eq!(
                    Some(String::from(TEST_VAL_2)),
                    back_to_irc.get_tag(TEST_KEY),
                    "Test key cannot be correctly retrieved after conversion \
                    and re-parsing.",
                );
                assert_eq!(
                    to_irc,
                    back_to_irc,
                    "After tag manipulation, Message produces a String which \
                    in turn DOES NOT produce an identical Message.",
                );
                assert_eq!(
                    from_irc,
                    back_from_irc,
                    "Strings from Messages are not consistent after tag \
                    manipulation.",
                );
            }
        }
    }
}
