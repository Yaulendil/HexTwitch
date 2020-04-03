/*
 * Module for the splitting of IRCv3 strings into a more usable format.
 */

use std::collections::HashMap;
use std::fmt;


// pub fn escape(line: &str) -> String {
//     line.replace(";", r"\:")
//         .replace(" ", r"\s")
//         .replace("\\", r"\\")
//         .replace("\n", r"\n")
//         .replace("\r", r"\n")
// }


pub fn unescape(line: &str) -> String {
    line.replace(r"\:", ";")
        .replace(r"\s", " ")
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\n", "\r")
}


/// Split a `str` at the first occurrence of another delimiting `str`.
pub fn split_at_first<'a>(line: &'a str, delim: &'a str) -> (&'a str, &'a str) {
    match line.find(delim) {
        Some(idx) => (&line[..idx], &line[idx + delim.len()..]),
        None => (line, ""),
    }
}


pub struct Author {
    pub host: String,
    pub user: String,
    pub nick: Option<String>,
}

impl Author {
    fn new(ustring: &str) -> Self {
        let nick: Option<String>;
        let userhost: &str;

        if ustring.contains('!') {
            let (n, uh): (&str, &str) = split_at_first(ustring, "!");

            nick = Some(String::from(n));
            userhost = uh;
        } else {
            nick = None;
            userhost = ustring;
        }

        let (user, host): (&str, &str) = split_at_first(userhost, "@");

        Self {
            host: String::from(host),
            user: String::from(user),
            nick,
        }
    }

    /// Retrieve the display name of the User represented by this `Author`. If
    ///     the User has a Nick, it will be returned; Otherwise, the Username is
    ///     returned.
    ///
    /// Return: `&str`
    pub fn display_name(&self) -> &str {
        if let Some(nick) = &self.nick {
            &nick
        } else {
            &self.user
        }
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(nick) = &self.nick {
            write!(f, "{}!{}@{}", nick, self.user, self.host)
        } else {
            write!(f, "{}@{}", self.user, self.host)
        }
    }
}


/// Message: An IRC Message in a usable structure.
///     author  : `Author`          : UserString: `[nick!]user@host`
///     command : `String`          : IRC Command.
///     args    : `Vec<String>`     : Arguments passed to the Command.
///     trail   : `String`          : Remainder of the Message. Whatever.
///     tags    : `Option<HashMap>` : IRCv3 Tags. Strings mapped to Strings.
///                                     This will be `None` if the original
///                                     message did not include a Tags segment.
pub struct Message {
    pub author: Author,
    pub command: String,
    pub args: Vec<String>,
    pub trail: String,
    pub tags: Option<HashMap<String, String>>,
}

impl Message {
    /// Split a raw IRCv3 `String` into a usable `Message`.
    ///
    /// Input: `&str`
    /// Return: `Message`
    pub fn new(full_str: &str) -> Self {
        let tags: Option<HashMap<String, String>>;
        let full_message: &str;
        //  "@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Break the line down.
        if full_str.starts_with('@') {
            //  The Tags String is the first half of the original message
            //      received by IRC. The "regular" message begins after the
            //      first space.
            let mut tagmap: HashMap<String, String> = HashMap::new();
            let (tag_str, main_str) = split_at_first(&full_str, " ");
            full_message = main_str;

            //  Break the tagstr into a Split Iterator. Spliterator?
            let tags_str_iter = tag_str[1..].split(';');

            //  Loop through the Spliterator of pair strings, and break each one
            //      the rest of the way down. Add values to the HashMap.
            for kvp in tags_str_iter {
                let (key, val) = split_at_first(kvp, "=");
                if !key.is_empty() { tagmap.insert(String::from(key), String::from(val)); }
            }
            tags = Some(tagmap);
        } else {
            tags = None;
            full_message = &full_str;
        }
        //  { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        //  ":asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Now, parse the message itself.
        //  This format is specified in Section 2.3.1 of RFC 1459.
        let prefix: &str;
        let message: &str;

        if full_message.starts_with(':') {
            //  This Message has a Prefix. The Prefix is most likely
            //      hostname and/or server info. It ends at the first space.
            let (p, m) = split_at_first(full_message, " ");
            prefix = &p[1..];
            message = m;
        } else {
            prefix = "";
            message = full_message;
        }
        //  "asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv"
        //  "PRIVMSG #zxcv arg2 :this is a message"

        //  The trailing data is found after a space and a colon. Everything up
        //      to that point is the IRC Command and any Arguments passed to it.
        let (cmd_and_args, trail) = split_at_first(message, " :");
        //  "PRIVMSG #zxcv arg2"
        //  "this is a message"

        //  The Command is the first word before any Arguments.
        let (command, args_str) = split_at_first(cmd_and_args, " ");
        //  "PRIVMSG"
        //  "#zxcv arg2"

        //  The Arguments should be split apart into a Vector of `String`s.
        let args = args_str.split_ascii_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();
        //  ["#zxcv", "arg2"]

        //  Compile everything into a Message Struct, and send it back up.
        Self {
            author: Author::new(prefix),  // "asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv"
            command: String::from(command),  // "PRIVMSG"
            args,  // ["#zxcv", "arg2"]
            trail: String::from(trail),  // "this is a message"
            tags,  // { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        }
    }

    /// Get a `String` representing this `Message` which will identify it.
    ///
    /// Return: `String`
    pub fn get_signature(&self) -> String {
        format!(
            "{}:{}",
            self.args.get(0).unwrap_or(&String::new()),
            self.author.user,
        )
    }

    /// Retrieve a Tag from the `Message`.
    ///
    /// Input: `&str`
    /// Return: `Option<String>`
    pub fn get_tag(&self, key: &str) -> Option<String> {
        self.tags.as_ref().and_then(|tags| Some(unescape(tags.get(key)?)))
    }
}

impl fmt::Display for Message {
    /// Format this `Message` into a format which is suitable for sending over
    ///     IRC.
    ///
    /// Return: `fmt::Result`
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(tags) = &self.tags {
            let tagline: &str = &tags.iter()
                .map(
                    |(key, val)|
                        if val.is_empty() {
                            String::from(key)
                        } else {
                            format!("{}={}", key, val)
                        }
                )
                .collect::<Vec<String>>()
                .join(";");
            write!(f, "@{} ", tagline)?;
        }

        write!(f, ":{} {}", self.author, self.command)?;
        for arg in &self.args { write!(f, " {}", arg)?; }
        if &self.trail != "" { write!(f, " :{}", self.trail)?; }

        Ok(())
    }
}
