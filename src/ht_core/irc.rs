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
//         .replace("\r", r"\r")
// }


/// Given a string which has had characters in it replaced with sanitized
///     stand-ins, replace the stand-ins with the special characters they
///     represent.
///
/// Input: `&str`
/// Return: `String`
pub fn unescape(line: &str) -> String {
    line.replace(r"\:", ";")
        .replace(r"\s", " ")
        .replace(r"\\", "\\")
        .replace(r"\n", "\n")
        .replace(r"\r", "\r")
}


/// Split a `str` at the first occurrence of another delimiting `str`.
pub fn split_at_first<'a>(line: &'a str, delim: &'a str) -> (&'a str, &'a str) {
    match line.find(delim) {
        Some(idx) => (&line[..idx], &line[idx + delim.len()..]),
        None => (line, ""),
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
    }
}

impl Prefix {
    /// Name: Return a representation of the author of a Message, intended for a
    ///     human to read. If the Prefix is a Server Name, it will be that; If
    ///     it is instead a User String, it will be the Nick.
    ///
    /// Return: `&str`
    pub fn name(&self) -> &str {
        #[allow(unused_variables)]
        match self {
            Prefix::ServerName(server) => { &server }
            Prefix::User { nick, user, host } => { &nick }
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Prefix::ServerName(server) => { write!(f, "{}", server) }
            Prefix::User { nick, user, host } => {
                write!(f, "{}", nick)?;
                if let Some(u) = user { write!(f, "!{}", u)?; }
                if let Some(h) = host { write!(f, "@{}", h)?; }
                Ok(())
            }
        }
    }
}

impl std::str::FromStr for Prefix {
    type Err = ();

    /// Split an IRC Prefix string into an Author.
    ///
    /// Input: `&str`
    /// Return: `Result<Prefix, ()>`
    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.contains('.') && !s.contains('@') {
            Ok(Prefix::ServerName(String::from(s)))
        } else {
            let user: Option<String>;
            let host: Option<String>;

            if s.contains('@') {
                let (a, b) = split_at_first(s, "@");
                s = a;
                host = Some(String::from(b));
            } else { host = None; }

            if s.contains('!') {
                let (a, b) = split_at_first(s, "!");
                s = a;
                user = Some(String::from(b));
            } else { user = None; }

            Ok(Prefix::User { nick: String::from(s), user, host })
        }
    }
}


/// Message: An IRC Message in a usable structure.
///     prefix  : `Prefix`          : UserString: `nick[!user][@host]`
///     command : `String`          : IRC Command.
///     args    : `Vec<String>`     : Arguments passed to the Command.
///     trail   : `String`          : Remainder of the Message. Whatever.
///     tags    : `Option<HashMap>` : IRCv3 Tags. Strings mapped to Strings.
///                                     This will be `None` if the original
///                                     message did not include a Tags segment.
#[derive(Debug, PartialEq)]
pub struct Message {
    pub prefix: Prefix,
    pub command: String,
    pub args: Vec<String>,
    pub trail: String,
    pub tags: Option<HashMap<String, String>>,
}

impl Message {
    /// Author: Return the name, put simply, of the source of this Message.
    ///
    /// Return: `&str`
    pub fn author(&self) -> &str { self.prefix.name() }

    /// Get a `String` representing this `Message` which will identify it.
    ///
    /// Return: `String`
    pub fn get_signature(&self) -> String {
        format!(
            "{}:{}",
            self.args.get(0).unwrap_or(&String::new()),
            self.author(),
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
            let mut tagseq = tags.iter().map(
                |(key, val)|
                    if val.is_empty() {
                        String::from(key)
                    } else {
                        format!("{}={}", key, val)
                    }
            ).collect::<Vec<String>>();

            tagseq.sort_unstable();
            write!(f, "@{} ", tagseq.join(";"))?;
        }

        write!(f, ":{} {}", self.prefix, self.command)?;
        for arg in &self.args { write!(f, " {}", arg)?; }
        if &self.trail != "" { write!(f, " :{}", self.trail)?; }

        Ok(())
    }
}

impl std::str::FromStr for Message {
    type Err = ();

    /// Split a raw IRC string into a usable `Message`.
    ///
    /// Input: `&str`
    /// Return: `Result<Message, ()>`
    fn from_str(full_str: &str) -> Result<Self, Self::Err> {
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
        Ok(Self {
            prefix: prefix.parse().unwrap(),  // "asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv"
            command: String::from(command),  // "PRIVMSG"
            args,  // ["#zxcv", "arg2"]
            trail: String::from(trail),  // "this is a message"
            tags,  // { badges: "bits/100", display-name: "AsdfQwert", emotes: "" }
        })
    }
}


#[cfg(test)]
mod tests_irc {
    use super::*;

    /// Test to confirm that converting back and forth between Message and text
    ///     will always produce the same results.
    #[test]
    fn irc_consistency() {
        for init in &[
            r"@badges=bits/100;display-name=AsdfQwert;emotes= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv PRIVMSG #zxcv arg2 :this is a message",
            r"@badges=subscriber/24,bits/1;emote-sets=0,2652,15749,19194,230961,320370,1228598;user-type= :tmi.twitch.tv USERSTATE #asdfqwert",
            r"@login=somejerk;room-id=;target-msg-id=15604c60-4d3b-8c1c-8e7a-c9ec2fb6c0cf;tmi-sent-ts=-6745368778951 :tmi.twitch.tv CLEARMSG #coolchannel :get a real job noob",
            r"@room-id=;target-user-id=8675309;tmi-sent-ts=1582958744397 :tmi.twitch.tv CLEARCHAT #coolchannel :somejerk",
            r"@badges=;color=#DABEEF;display-name=Asdf\sQwert;emotes=;message-id=2;thread-id=1337-9001;turbo=0;user-id=123456789;user-type= :asdfqwert!asdfqwert@asdfqwert.tmi.twitch.tv WHISPER thyself :asdf"
        ] {
            let to_irc: Message = init.parse().expect("Failed to parse initial string.");
            let from_irc: String = to_irc.to_string();
            let back_to_irc: Message = init.parse().expect("Failed to re-parse second string.");
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
}
