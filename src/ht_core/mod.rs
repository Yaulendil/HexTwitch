mod ircv3;
mod events;
pub mod printing;


use std::mem::drop;

use chrono::{DateTime, Utc};
use hexchat::{EatMode, get_channel_name, get_network_name, PrintEvent, strip_formatting};
use parking_lot::Mutex;

use ircv3::Message;
use printing::{Badges, echo, EVENT_ERR, USERSTATE, WHISPER_SIDES};


pub(crate) struct Sponge {
    pub signature: Option<String>,
    pub value: Option<Message>,
}

impl Sponge {
    /// Place a Message into the Sponge. The previous Message in the Sponge, if
    ///     any, is removed. Takes Ownership of the new Message.
    ///
    /// Input: `Message`
    pub fn put(&mut self, new: Message) {
        self.signature.replace(new.get_signature());
        self.value.replace(new);
    }

    /// Remove the Message from the Sponge, but only if the current Message has
    ///     the Signature specified.
    ///
    /// Input: `&str`
    /// Return: `Option<Message>`
    pub fn pop(&mut self, signature: &str) -> Option<Message> {
        match (&self.signature, &mut self.value) {
            (Some(sig), msg) if sig == signature => msg.take(),
            _ => None,
        }
    }
}


safe_static! {
    static lazy CURRENT: Mutex<Sponge> = Mutex::new(Sponge { signature: None, value: None });
}


/// Reset the Color of a newly-focused Tab.
pub(crate) fn cb_focus(_etype: PrintEvent, _word: &[String]) -> EatMode {
    EatMode::None
}


/// Hide a Join Event if it is fake.
pub(crate) fn cb_join(_etype: PrintEvent, _word: &[String]) -> EatMode {
    EatMode::None
}


pub(crate) fn cb_print(etype: PrintEvent, word: &[String]) -> EatMode {
    let channel = get_channel_name();
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let sig: &str = &format!(
                "{}:{}",
                &channel,
                strip_formatting(&word[0]).unwrap_or_default(),
            );

            let mut guard = CURRENT.lock();
            let msg_maybe = guard.pop(sig);
            drop(guard);

            if let Some(msg) = msg_maybe {
                //  TODO: Re-emit Print with User Badges, etc.

                match etype {
                    PrintEvent::YOUR_MESSAGE
                    | PrintEvent::YOUR_ACTION
                    => {
                        let badge_str: String = USERSTATE.read().get(&channel).to_string();
                        echo(etype, &[&word[0] as &str, &word[1], "_", &badge_str]);

                        EatMode::All
                    }
                    PrintEvent::CHANNEL_MESSAGE
                    | PrintEvent::CHANNEL_ACTION
                    | PrintEvent::CHANNEL_MSG_HILIGHT
                    | PrintEvent::CHANNEL_ACTION_HILIGHT
                    => {
                        let badges = Badges::new(
                            &msg.get_tag("badges").unwrap_or_default()
                        );
                        echo(
                            etype,
                            &[&word[0] as &str, &word[1], "", &badges.output],
                        );

                        EatMode::All
                    }
                    _ => EatMode::None
                }
            } else if let PrintEvent::YOUR_MESSAGE | PrintEvent::YOUR_ACTION = etype {
                if channel.starts_with(&WHISPER_SIDES)
                    && channel.ends_with(&WHISPER_SIDES) {
                    //  User has spoken inside a Whisper Tab. We must take
                    //      the message typed, and forward it to the Whisper
                    //      Command via ".w {}".
                    //  TODO
                    EatMode::None

                } else if &word[2] == "" {
                    //  User has spoken in a normal Channel, but has no Badges.
                    //      Add the Badges from the User State and re-emit.
                    let badge_str: String = USERSTATE.read().get(&channel).to_string();
                    echo(etype, &[&word[0] as &str, &word[1], "_", &badge_str]);

                    EatMode::All

                } else {
                    //  This is a re-emit. Do nothing.
                    EatMode::None
                }
            } else { EatMode::None }
        }
        _ => EatMode::None,
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub(crate) fn cb_server(word: &[String], _dt: DateTime<Utc>, raw: String) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let msg: Message = Message::new(&raw);
            let opt_eat: Option<EatMode> = match msg.command.as_str() {
                //  Chat Messages.
                "PRIVMSG" => {
                    let mut guard = CURRENT.lock();
                    guard.put(msg);
                    drop(guard);

                    Some(EatMode::None)
                }
                "WHISPER" => events::whisper(msg),

                "ROOMSTATE" => Some(EatMode::Hexchat),
                "USERSTATE" => events::userstate(msg),

                "USERNOTICE" => events::usernotice(msg),
                "HOSTTARGET" => events::hosttarget(&word[3][1..]),

                //  Moderator Actions.
                "CLEARMSG" => events::clearmsg(msg),
                "CLEARCHAT" => events::clearchat(msg),
                _ => Some(EatMode::None),
            };

            if let Some(eat) = opt_eat { eat } else {
                echo(
                    EVENT_ERR,
                    &[&raw],
                );
                EatMode::None
            }
        }
        _ => EatMode::None,
    }
}


pub(crate) fn cmd_title(_arg: &[String]) -> EatMode {
    EatMode::None
}


pub(crate) fn cmd_tjoin(_arg: &[String]) -> EatMode {
    EatMode::None
}
