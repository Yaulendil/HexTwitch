/*
 * Module for the splitting of IRCv3 strings into a more usable format.
 */

use std::collections::HashMap;
use std::str;


/// Split a `str` at the first occurrence of another delimiting `str`.
pub fn split_at_first<'a>(line: &'a str, delim: &'a str) -> (&'a str, &'a str) {
    match line.find(delim) {
        Some(idx) => (&line[..idx], &line[idx + delim.len()..]),
        None => (line, ""),
    }
}


pub struct Message {
    pub ustring: String,
    pub command: String,
    pub args: Vec<String>,
    pub trail: String,
    pub tags: HashMap<String, String>,
}

impl Message {
    /// Split a raw IRCv3 line into usable data.
    ///
    /// Provided a String in IRCv3 format, break it down into a Message Struct.
    /// Message Struct contains:
    ///     ustring : `String`        : UserString: `[nick!]user@host`
    ///     command : `String`        : IRC Command.
    ///     args    : `Vec<String>`   : Arguments passed to the Command.
    ///     trail   : `String`        : Remainder of the Message. Whatever.
    ///     tags    : `HashMap`       : IRCv3 Tags. Strings mapped to Strings.
    ///
    /// Input: `&str`
    /// Return: `Message`
    pub fn new(full_str: &str) -> Self {
        let mut tags = HashMap::new();
        let message: &str;
        //  "@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Break the line down.
        if full_str.starts_with('@') {
            //  The Tags String is the first half of the original message
            //      received by IRC. The "regular" message begins after the
            //      first space.
            let (tag_str, main_str) = split_at_first(&full_str, " ");
            message = main_str;

            //  Break the tagstr into a Split Iterator. Spliterator?
            let tags_str_iter = tag_str[1..].split(';');

            //  Loop through the Spliterator of pair strings, and break each one
            //      the rest of the way down. Add values to the HashMap.
            for kvp in tags_str_iter {
                let (key, val) = split_at_first(kvp, "=");
                if !key.is_empty() { tags.insert(key.to_string(), val.to_string()); }
            }
        } else { message = &full_str; }
        //  { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        //  ":asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message"

        //  Now, parse the message itself.
        //  This format is specified in Section 2.3.1 of RFC 1459.
        let prefix: &str = {
            if message.starts_with(':') {
                //  This Message has a Prefix. The Prefix is most likely
                //      hostname and/or server info. It ends at the first space.
                &split_at_first(&message, " ").0[1..]
            } else { "" }
        };
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

    /// Get a `String` representing this `Message` which will identify it.
    ///
    /// Return: `String`
    pub fn get_signature(&mut self) -> String {
        format!("{}:{}", self.ustring, self.command)
        //  TODO: Change to something useful.
    }
}