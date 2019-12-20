/*
 * Module for the splitting of IRCv3 strings into a more usable format.
 */

use std::collections::HashMap;
use std::str;


pub struct Message<'a> {
    prefix: &'a str,
    command: &'a str,
    args: Vec<&'a str>,
    trail: &'a str,
    tags: HashMap<&'a str, &'a str>,
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
pub fn split<'a>(line: &'a mut String) -> Message<'a> {
    // First, decode the data into something we can work.
    let full_str: &str = line.as_str();

    // Then, initialize the Output Structures.
    let mut tags = HashMap::new();
    let message: &str;

    // Third, break the line down.
    if full_str.starts_with('@') {
        // The Tags String is the first half of the original message received by IRC. The "regular"
        //  message begins after the first space.
        let (tag_str, msg_str) = split_at_first(&full_str[1..], " ");
        message = msg_str;

        // Break the tagstr into a Split Iterator. Spliterator?
        let tags_str_iter = tag_str.split(';');

        // Loop through the Spliterator of pair strings, and break each one the
        //  rest of the way down. Add values to the HashMap.
        for kvp in tags_str_iter {
            if !kvp.is_empty() {
                let (key, val) = split_at_first(kvp, "=");
                if !key.is_empty() {
                    tags.insert(key, val);
                }
            }
        }
    } else {
        // There are no tags. This is pure message.
        message = full_str;
    }

    // Now, parse the message itself.
    // This format is specified in Section 2.3.1 of RFC 1459.
    let prefix: &str;
    if message.starts_with(':') {
        // This Message has a Prefix. The Prefix is most likely hostname and/or
        //  server info. It ends at the first space.
        prefix = split_at_first(&message[1..], " ").0;
    } else {
        prefix = ""
    }

    // The trailing data is found after a space and a colon. Everything up to
    //  that point is the IRC Command and any Arguments passed to it.
    let (cmd_and_args, trail) = split_at_first(message, " :");

    // The Command is the first word before any Arguments.
    let (command, args_str) = split_at_first(cmd_and_args, " ");

    // The Arguments should be split apart into a Vector of `str`s.
    let args = args_str.split_ascii_whitespace().collect::<Vec<&'a str>>();

    // Compile everything into a Message Struct, and send it back up.
    let output = Message {
        prefix,
        command,
        args,
        trail,
        tags,
    };
    output
}
