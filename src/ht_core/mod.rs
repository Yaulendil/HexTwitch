mod ircv3;
mod events;
pub mod printing;
mod tabs;


use std::mem::drop;

use chrono::{DateTime, Utc};
use hexchat::{ChannelRef, EatMode, get_channel_name, get_network_name, PrintEvent, send_command, strip_formatting};
use parking_lot::Mutex;

use ircv3::Message;
use printing::{
    Badges,
    echo,
    EVENT_ALERT,
    EVENT_ERR,
    EVENT_REWARD,
    USERSTATE,
    WHISPER_SIDES,
};
use tabs::TABCOLORS;


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


/// Message comes from Server. IRC Representation available.
fn print_with_irc(channel: &str, etype: PrintEvent, word: &[String], msg: Message) -> EatMode {
    if let Some(tags) = &msg.tags {
        if let Some(bits) = tags.get("bits") {
            if let Ok(n) = bits.parse::<usize>() {
                events::cheer(&msg.author.display_name(), n);
            }
        }

        if let Some(custom) = tags.get("custom-reward-id") {
            //  This Message is a Custom Reward.
            if let Some(notif) = events::reward(custom) {
                //  We know what it should be called.
                echo(
                    EVENT_REWARD,
                    &[
                        notif,
                        &format!("{}:", &msg.author.display_name()),
                        &word[1].as_str(),
                    ],
                    2,
                );
            } else {
                //  We do NOT know what it should be called. Use a
                //      generic "CUSTOM" label, and also print the
                //      ID.
                echo(
                    EVENT_REWARD,
                    &[
                        "CUSTOM",
                        &format!("({}) {}:", custom, &msg.author.display_name()),
                        &word[1].as_str(),
                    ],
                    2,
                );
            }

            return EatMode::All;
        } else if tags.get("msg-id")
            .unwrap_or(&String::new())
            == "highlighted-message" {
            echo(
                EVENT_ALERT,
                &[
                    &msg.author.display_name(),
                    &word[1].as_str(),
                ],
                2,
            );

            return EatMode::All;
        }
    }

    match etype {
        PrintEvent::YOUR_MESSAGE
        | PrintEvent::YOUR_ACTION
        => {
            let badge_str: String = USERSTATE.read().get(&channel).to_string();
            echo(etype, &[&word[0] as &str, &word[1], "_", &badge_str], 2);

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
                {
                    if etype == PrintEvent::CHANNEL_MSG_HILIGHT
                        || etype == PrintEvent::CHANNEL_ACTION_HILIGHT
                    { 3 } else { 2 }
                },
            );

            send_command(&format!(
                "RECV :{0}!{0}@twitch.tv/{0} JOIN {1}",
                msg.author.user.to_ascii_lowercase(),
                channel,
            ));

            EatMode::All
        }
        _ => EatMode::None
    }
}


/// No IRC Representation available for Message.
fn print_without_irc(channel: &str, etype: PrintEvent, word: &[String]) -> EatMode {
    if !channel.starts_with("#") {
        //  User has spoken inside a Whisper Tab. We must take
        //      the message typed, and forward it to the Whisper
        //      Command via ".w {}".
        events::whisper_send(etype, &channel, word);

        EatMode::All
    } else if &word[2] == "" {
        //  User has spoken in a normal Channel, but has no Badges.
        //      Add the Badges from the User State and re-emit.
        let badge_str: String = USERSTATE.read().get(&channel).to_string();
        echo(etype, &[&word[0] as &str, &word[1], "_", &badge_str], 2);

        EatMode::All
    } else {
        //  This is a re-emit. Do nothing.
        EatMode::None
    }
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
            let msg: Message = Message::new(&raw);
            let opt_eat: Option<EatMode> = match msg.command.as_str() {
                //  Chat Messages.
                "PRIVMSG" => {
                    let mut guard = CURRENT.lock();
                    guard.put(msg);
                    drop(guard);

                    Some(EatMode::None)
                }
                "WHISPER" => events::whisper_recv(msg),

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
                    1,
                );
                EatMode::None
            }
        }
        _ => EatMode::None,
    }
}


pub(crate) fn cmd_title(arg: &[String]) -> EatMode {
    send_command(&format!(
        "RECV :Twitch@twitch.tv TOPIC #{} :{}",
        &arg[1].to_ascii_lowercase(),
        &arg[2..].join(" "),
    ));

    EatMode::All
}


pub(crate) fn cmd_tjoin(arg: &[String]) -> EatMode {
    send_command(&format!(
        "JOIN {}",
        &arg[1..].join(" "),
    ));

    EatMode::All
}
