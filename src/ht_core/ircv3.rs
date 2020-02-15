/*
 * Module for the splitting of IRCv3 strings into a more usable format.
 */

use std::collections::HashMap;
use std::str;


pub fn escape(line: &str) -> String {
    line.replace(";", r"\:")
        .replace(" ", r"\s")
        .replace("\\", r"\\")
        .replace("\n", r"\n")
        .replace("\r", r"\n")
}


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


/// Message: An IRC Message in a usable structure.
///     ustring : `String`          : UserString: `[nick!]user@host`
///     command : `String`          : IRC Command.
///     args    : `Vec<String>`     : Arguments passed to the Command.
///     trail   : `String`          : Remainder of the Message. Whatever.
///     tags    : `Option<HashMap>` : IRCv3 Tags. Strings mapped to Strings.
///                                     This will be `None` if the original
///                                     message did not include a Tags segment.
pub struct Message {
    pub ustring: String,
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
                if !key.is_empty() { tagmap.insert(key.to_string(), val.to_string()); }
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
            let (p, m) = &split_at_first(full_message, " ");
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
            ustring: prefix.to_string(),  // "asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv"
            command: command.to_string(),  // "PRIVMSG"
            args,  // ["#zxcv", "arg2"]
            trail: trail.to_string(),  // "this is a message"
            tags,  // { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        }
    }

    /// Convert this `Message` into a `String` which is suitable for sending
    ///     over IRC.
    ///
    /// Return: `String`
    pub fn as_str(&self) -> String {
        let mut out = String::new();
        if let Some(tags) = &self.tags {
            out.push('@');
            out.push_str(&tags.iter()
                .map(|(key, val)|
                    if val == "" {
                        key.to_owned()
                    } else {
                        format!("{}={}", key, val)
                    })
                .collect::<Vec<String>>().join(";"));
            out.push(' ');
        }
        out.push(':');
        out.push_str(&self.ustring);
        out.push(' ');
        out.push_str(&self.command);

        for arg in &self.args {
            out.push(' ');
            out.push_str(arg);
        }
        if &self.trail != "" {
            out.push_str(" :");
            out.push_str(&self.trail);
        }

        out
    }

    /// Get a `String` representing this `Message` which will identify it.
    ///
    /// Return: `String`
    pub fn get_signature(&self) -> String {
        format!("{}:{}", self.ustring, self.command)
        //  TODO: Change to something useful.
    }

    /// Retrieve a Tag from the `Message`.
    ///
    /// Input: `&str`
    /// Return: `Option<String>`
    pub fn get_tag(&self, key: &str) -> Option<String> {
        self.tags.as_ref().and_then(|tags| Some(unescape(tags.get(key)?)))
    }

    /// Retrieve the name of the User who sent the `Message`. If the User has a
    ///     Nick, it will be returned; Otherwise, the Username is returned.
    ///
    /// Return: `&str`
    pub fn get_user(&self) -> &str {
        split_at_first(
            &self.ustring,
            if self.ustring.contains('!') { "!" } else { "@" }
        ).0
    }
}
