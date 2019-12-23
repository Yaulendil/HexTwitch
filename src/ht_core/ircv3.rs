/*
 * Module for the splitting of IRCv3 strings into a more usable format.
 */

use std::collections::HashMap;
use std::str;


pub struct Message {
    pub prefix: String,
    pub command: String,
    pub args: Vec<String>,
    pub trail: String,
    pub tags: HashMap<String, String>,
}


impl Message {
    pub fn get_signature(&mut self) -> String {
        format!("{}:{}", self.prefix, self.command)
        //  TODO: Change to something useful.
    }
}


/// Split a `str` at the first occurrence of another delimiting `str`.
fn split_at_first<'a>(line: &'a str, delim: &'a str) -> (&'a str, &'a str) {
    match line.find(delim) {
        Some(idx) => (&line[..idx], &line[idx + delim.len()..]),
        None => (line, ""),
    }
}


/// Split a raw IRCv3 line into usable data.
///
/// Provided a String in IRCv3 format, break it down into a Message Struct.
/// Message Struct contains:
///     prefix      : str       : Message Prefix; probably Hostname/Server data.
///     command     : str       : IRC Command.
///     arguments   : Vec<str>  : Arguments passed to the Command.
///     trail       : str       : Remainder of the Message. Whatever.
///     tags        : HashMap   : IRCv3 Tags. Strings mapped to Strings.
///
/// Input: `String`
/// Return: `Message`
pub fn split(full_str: &str) -> Message {
    let mut tags = HashMap::new();
    let message: &str;

    //  Break the line down.
    if full_str.starts_with('@') {
        //  The Tags String is the first half of the original message received by IRC. The "regular"
        //      message begins after the first space.
        let (tag_str, main_str) = split_at_first(&full_str, " ");
        message = main_str;

        //  Break the tagstr into a Split Iterator. Spliterator?
        let tags_str_iter = tag_str[1..].split(';');

        //  Loop through the Spliterator of pair strings, and break each one the
        //      rest of the way down. Add values to the HashMap.
        for kvp in tags_str_iter {
            let (key, val) = split_at_first(kvp, "=");
            if !key.is_empty() {
                tags.insert(key.to_string(), val.to_string());
            }
        }
    } else {
        //  There are no tags. This is pure message.
        message = &full_str;
    }

    //  Now, parse the message itself.
    //  This format is specified in Section 2.3.1 of RFC 1459.
    let prefix: &str = {
        if message.starts_with(':') {
            //  This Message has a Prefix. The Prefix is most likely hostname and/or server info. It
            //      ends at the first space.
            &split_at_first(&message, " ").0[1..]
        } else {
            ""
        }
    };

    //  The trailing data is found after a space and a colon. Everything up to
    //      that point is the IRC Command and any Arguments passed to it.
    let (cmd_and_args, trail) = split_at_first(message, " :");

    //  The Command is the first word before any Arguments.
    let (command, args_str) = split_at_first(cmd_and_args, " ");

    //  The Arguments should be split apart into a Vector of `String`s.
    let args = args_str.split_ascii_whitespace().map(String::from).collect::<Vec<String>>();

    //  Compile everything into a Message Struct, and send it back up.
    Message {
        prefix: prefix.to_string(),
        command: command.to_string(),
        args,
        trail: trail.to_string(),
        tags,
    }
}
