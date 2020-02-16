mod ircv3;
mod events;
mod printing;


use chrono::{DateTime, Utc};
use hexchat::{EatMode, get_channel_name, get_network_name, PrintEvent, strip_formatting};
use parking_lot::Mutex;

use ircv3::Message;
use printing::{echo, EVENT_ERR, WHISPER_SIDES};


pub struct Sponge {
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


pub fn cb_print(etype: PrintEvent, word: &[String]) -> EatMode {
    let channel = get_channel_name();
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let sig: &str = &format!(
                "{}:{}",
                &channel,
                strip_formatting(&word[0]).unwrap_or_default()
            );

            if let Some(msg) = CURRENT.lock().pop(sig) {
                //  TODO: Re-emit Print with User Badges, etc.

                match etype {
                    PrintEvent::YOUR_MESSAGE | PrintEvent::YOUR_ACTION if {
                        channel.starts_with(&WHISPER_SIDES)
                            && channel.ends_with(&WHISPER_SIDES)
                    } => {
                        //  User has spoken inside a Whisper Tab. We must take
                        //      the message typed, and forward it to the Whisper
                        //      Command via ".w {}".
                        //  TODO
                    }
                    PrintEvent::CHANNEL_MESSAGE => {}
                    PrintEvent::CHANNEL_ACTION => {}
                    PrintEvent::CHANNEL_MSG_HILIGHT => {}
                    PrintEvent::CHANNEL_ACTION_HILIGHT => {}
                    PrintEvent::YOUR_MESSAGE => {}
                    PrintEvent::YOUR_ACTION => {}
                    _ => return EatMode::None,
                }

                EatMode::All
            } else { EatMode::None }
        }
        _ => EatMode::None,
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub fn cb_server(word: &[String], dt: DateTime<Utc>, raw: String) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let msg: Message = Message::new(&raw);
            let opt_eat: Option<EatMode> = match msg.command.as_str() {
                //  Chat Messages.
                "PRIVMSG" => {
                    CURRENT.lock().put(msg);
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
