mod events;
mod irc;
mod output;


use std::mem::drop;

use chrono::{DateTime, Utc};
use hexchat::{
    ChannelRef,
    EatMode,
    delete_pref,
    get_channel_name,
    get_network_name,
    get_pref_string,
    get_prefs,
    PrintEvent,
    send_command,
    set_pref_string,
    strip_formatting,
};
use parking_lot::Mutex;

pub use events::ensure_tab;
use irc::Message;
use output::{
    echo,
    EVENT_ERR,
    EVENT_NORMAL,
    print_with_irc,
    print_without_irc,
    TABCOLORS,
    USERSTATE,
};


struct Sponge {
    signature: Option<String>,
    value: Option<Message>,
}

impl Sponge {
    /// Place a Message into the Sponge. The previous Message in the Sponge, if
    ///     any, is removed. Takes Ownership of the new Message.
    ///
    /// Input: `Message`
    fn put(&mut self, new: Message) {
        self.signature.replace(new.get_signature());
        self.value.replace(new);
    }

    /// Remove the Message from the Sponge, but only if the current Message has
    ///     the Signature specified.
    ///
    /// Input: `&str`
    /// Return: `Option<Message>`
    fn pop(&mut self, signature: &str) -> Option<Message> {
        match (&self.signature, &mut self.value) {
            (Some(sig), msg) if sig == signature => msg.take(),
            _ => None,
        }
    }
}


safe_static! {
    static lazy CURRENT: Mutex<Sponge> = Mutex::new(Sponge { signature: None, value: None });
}


fn check_message(channel: &str, author: &str) -> Option<Message> {
    let sig: &str = &format!(
        "{}:{}",
        &channel,
        strip_formatting(author).unwrap_or_default(),
    );

    let mut guard = CURRENT.lock();
    let msg_maybe = guard.pop(sig);
    drop(guard);

    msg_maybe
}


/// Reset the Color of a newly-focused Tab.
pub(crate) fn cb_focus(_channel: ChannelRef) -> EatMode {
    TABCOLORS.write().reset();
    EatMode::None
}


/// Hide a Join Event if it is fake.
pub(crate) fn cb_join(_etype: PrintEvent, word: &[String]) -> EatMode {
    if get_network_name()
        .unwrap_or_default()
        .eq_ignore_ascii_case("twitch")
        && !word[2].contains("tmi.twitch.tv")
    {
        EatMode::All
    } else {
        EatMode::None
    }
}


pub(crate) fn cb_print(etype: PrintEvent, word: &[String]) -> EatMode {
    let channel = get_channel_name();
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            if let Some(msg) = check_message(&channel, &word[0]) {
                //  Message comes from Server. IRC Representation available.
                print_with_irc(&channel, etype, word, msg)
            } else if let PrintEvent::YOUR_MESSAGE | PrintEvent::YOUR_ACTION = etype {
                //  No IRC Representation available for Message.
                print_without_irc(&channel, etype, word)
            } else {
                EatMode::None
            }
        }
        _ => EatMode::None
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub(crate) fn cb_server(word: &[String], _dt: DateTime<Utc>, raw: String) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let msg: Message = raw.parse().expect("Failed to parse IRC Message");
            let opt_eat: Option<EatMode> = match &*msg.command {
                //  Chat Messages.
                "PRIVMSG" => {
                    let mut guard = CURRENT.lock();
                    guard.put(msg);
                    drop(guard);

                    Some(EatMode::None)
                }
                "WHISPER" => events::whisper_recv(msg),

                "ROOMSTATE" => Some(EatMode::Hexchat),
                "USERSTATE" => {
                    USERSTATE.write().set(
                        get_channel_name(),
                        msg.get_tag("badges").unwrap_or_default(),
                    );
                    Some(EatMode::All)
                },

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
                    1,
                );
                EatMode::None
            }
        }
        _ => EatMode::None,
    }
}


pub(crate) fn cmd_reward(argslice: &[String]) -> EatMode {
    let mut arg: Vec<&str> = Vec::new();
    for a in argslice[1..].iter() {
        if !a.is_empty() { arg.push(&*a); }
    }
    let len = arg.len();

    if len < 1 {
        //  Print the current Reward Names.
        echo(EVENT_NORMAL, &["REWARD EVENTS:"], 0);
        for pref in get_prefs() {
            if !pref.is_empty() {
                echo(
                    EVENT_NORMAL,
                    &[format!(
                        "{}: '{}'",
                        pref,
                        get_pref_string(&pref)
                            .unwrap_or_default(),
                    )],
                    0,
                );
            }
        }
    } else {
        if let Ok(_) = {
            if len < 2 {
                //  Unset a Reward.
                delete_pref(&arg[0].to_lowercase())
            } else {
                //  Set a Reward.
                set_pref_string(
                    &arg[0].to_lowercase(),
                    &arg[1..].join(" ").trim()
                )
            }
        } {
            echo(EVENT_NORMAL, &["Preference set."], 0);
        } else {
            echo(EVENT_ERR, &["FAILED to set Preference."], 0);
        }
    }

    EatMode::All
}


pub(crate) fn cmd_title(arg: &[String]) -> EatMode {
    send_command(&format!(
        "RECV :Twitch@twitch.tv TOPIC #{} :{}",
        &arg[1].to_ascii_lowercase(),
        &arg[2..].join(" ").trim(),
    ));

    EatMode::All
}


pub(crate) fn cmd_tjoin(arg: &[String]) -> EatMode {
    send_command(&format!(
        "JOIN {}",
        &arg[1..].join(" ").trim(),
    ));

    EatMode::All
}
