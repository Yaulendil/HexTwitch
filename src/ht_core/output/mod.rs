pub(super) mod channels;
pub(super) mod prediction;
mod printing;
mod statics;
mod tabs;

use std::borrow::Cow;
use hexchat::{EatMode, print_plain, PrintEvent};
use crate::{irc::{Message, Prefix}, prefs::{HexPref, PREF_ANNOUNCE}};
use super::{events, ignore_next_print_event};
pub use printing::{
    alert_basic,
    alert_error,
    alert_subscription,
    alert_sub_upgrade,
    badge_parse,
    Badges,
    change_topic,
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_ERR,
    EVENT_NORMAL,
    EVENT_REWARD,
};
pub use statics::{BADGES_UNKNOWN, CHANNELS, TABCOLORS, USERSTATE};
pub use tabs::TabColor;


pub const FAKE_MODE_NAME: &str = "HexTwitch";


#[cfg(feature = "fake-joins")]
pub(super) fn fake_join(channel: &str, user: &str) {
    if user != hexchat::get_nickname() {
        cmd!(
            "RECV :{user}!twitch.tv/{user} JOIN {channel}",
            channel = channel,
            user = user,
        );
    }
}


#[cfg(feature = "fake-modes")]
pub(super) fn fake_mode(channel: &str, user: &str, op: bool) {
    cmd!(
        "RECV :{by} MODE {channel} {mode} {user}",
        by = FAKE_MODE_NAME,
        mode = if op { "+o" } else { "-o" },
        channel = channel,
        user = user,
    );
}


enum AnnounceType {
    /// Command: /announce (via IRC)
    None,
    /// Command: /announce (via website)
    Primary,
    /// Command: /announceblue
    Blue,
    /// Command: /announcegreen
    Green,
    /// Command: /announceorange
    Orange,
    /// Command: /announcepurple
    Purple,
    Unknown(String),
}

impl AnnounceType {
    fn from_tag(param: Option<String>) -> Self {
        match param {
            None => Self::None,
            Some(color) => match color.as_str() {
                "PRIMARY" => Self::Primary,
                "BLUE" => Self::Blue,
                "GREEN" => Self::Green,
                "ORANGE" => Self::Orange,
                "PURPLE" => Self::Purple,
                _ => Self::Unknown(color),
            }
        }
    }

    const fn color(&self) -> &'static str {
        match self {
            Self::Blue => "\x0311",
            Self::Green => "\x0309",
            Self::Orange => "\x0307",
            Self::Purple => "\x0313",
            _ => "\x0300",
        }
    }

    fn mode(&self) -> Cow<'static, str> {
        match self {
            Self::Unknown(color) => Cow::Owned(format!("![{}] ", color)),
            Self::Primary => Cow::Borrowed("[WEB] "),
            Self::None => Cow::Borrowed("[IRC] "),

            Self::Blue => Cow::Borrowed("[B] "),
            Self::Green => Cow::Borrowed("[G] "),
            Self::Orange => Cow::Borrowed("[O] "),
            Self::Purple => Cow::Borrowed("[P] "),
            // _ => Cow::Borrowed(""),
        }
    }
}


pub fn print_announcement(mut msg: Message) -> Option<EatMode> {
    let author = msg.get_tag("login")?;
    let color = AnnounceType::from_tag(msg.get_tag("msg-param-color"));

    //  Change the message to a `PRIVMSG` and pretend to receive it anew. This
    //      will cause the plugin to properly process it, and then, back in this
    //      function, we will generate a fancy colored representation.
    msg.command = String::from("PRIVMSG");
    msg.prefix = Prefix::User {
        nick: author.clone(),
        user: None,
        host: None,
    };
    cmd!("RECV {}", msg);

    if PREF_ANNOUNCE.is(&true) {
        //  If the announcement content was a `/me` invocation, it must be
        //      extracted from the `ACTION` frame and presented differently.
        let (content, is_me) = match msg.trail.strip_prefix("\x01ACTION ") {
            Some(action_x01) => match action_x01.strip_suffix('\x01') {
                Some(action) => (action, true),
                None => (msg.trail.as_str(), false),
            }
            None => (msg.trail.as_str(), false),
        };

        print_plain(&format!(
            "\x0302{mode}\x02{color}{text}\x0F",
            color = color.color(),
            mode = color.mode(),
            text = if is_me {
                Cow::Owned(format!("\x1D{} \x02{}", author, content))
            } else {
                Cow::Borrowed(content)
            },
        ));
    }

    Some(EatMode::All)
}


/// Message comes from Server. IRC Representation available.
pub fn print_with_irc(
    channel: &str,
    etype: PrintEvent,
    word: &[String],
    msg: Message,
) -> EatMode {
    let author: &str = msg.author();

    if msg.has_tags() {
        if let Some(bits) = msg.get_tag("bits") {
            if let Ok(n) = bits.parse::<usize>() {
                events::cheer(author, n);
            }
        }

        if let Some(eat) = events::reward(word, &msg) {
            return eat;
        }
    }

    match etype {
        PrintEvent::YOUR_MESSAGE
        | PrintEvent::YOUR_ACTION
        => {
            let badges;
            let state;
            let bstr = match msg.get_tag("badges") {
                Some(tag) => {
                    badges = badge_parse(
                        tag,
                        msg.get_tag("badge-info").unwrap_or_default(),
                    );
                    badges.as_str()
                }
                None => {
                    state = USERSTATE.get(&channel);
                    &state
                }
            };

            ignore_next_print_event();
            echo(etype, &[
                word[0].as_str(), // Name
                word[1].as_str(), // Text
                bstr, // Mode
                word[3].as_str(), // "Identified text"
            ], TabColor::Message);

            EatMode::All
        }
        PrintEvent::CHANNEL_MESSAGE
        | PrintEvent::CHANNEL_ACTION
        | PrintEvent::CHANNEL_MSG_HILIGHT
        | PrintEvent::CHANNEL_ACTION_HILIGHT
        => {
            let badges: Badges = badge_parse(
                msg.get_tag("badges").unwrap_or_default(),
                msg.get_tag("badge-info").unwrap_or_default(),
            );
            let color: TabColor = match etype {
                PrintEvent::CHANNEL_ACTION_HILIGHT
                | PrintEvent::CHANNEL_MSG_HILIGHT
                => TabColor::Highlight,
                _ => TabColor::Message,
            };

            if msg.get_tag("anonymous-cheerer").is_none() {
                #[cfg(feature = "fake-joins")]
                fake_join(channel, author);
                #[cfg(feature = "fake-modes")]
                fake_mode(channel, author, badges.is_op());
            }

            ignore_next_print_event();
            echo(etype, &[
                word[0].as_str(), // Name
                word[1].as_str(), // Text
                badges.as_str(), // Mode
                word[3].as_str(), // "Identified text"
            ], color);

            badges.update_prediction(&channel);

            EatMode::All
        }
        _ => EatMode::None,
    }
}


/// No IRC Representation available for Message.
pub fn print_without_irc(channel: &str, etype: PrintEvent, word: &[String]) -> EatMode {
    if word[1].starts_with(".w ") || word[1].starts_with("/w ") {
        //  User has executed `.w`. We must take the message typed, and forward
        //      it to the Whisper Handler.
        events::whisper_send_command(etype, &channel, word);

        EatMode::All
    } else if !channel.starts_with::<&[char]>(&['#', '&']) {
        //  User has spoken inside a Whisper Tab. We must take the message
        //      typed, and forward it to the Whisper Handler.
        events::whisper_send_channel(etype, &channel, word);

        EatMode::All
    } else if let Some(cmd) = twitch_command(&word[1]) {
        echo(
            PrintEvent::NOTICE_SEND,
            &[channel, &format!("/{}", cmd)],
            TabColor::None,
        );

        EatMode::All
    } else {
        //  User has spoken in a normal Channel, but has not yet been given
        //      Badges. Add the Badges from the User State and re-emit.
        ignore_next_print_event();
        echo(etype, &[
            word[0].as_str(), // Name
            word[1].as_str(), // Text
            &USERSTATE.get(channel), // Mode
            word[3].as_str(), // "Identified text"
        ], TabColor::Message);

        EatMode::All
    }
}


fn twitch_command(line: &str) -> Option<&str> {
    match line.as_bytes() {
        [] => None,
        [b'.', b'.', ..] => None,
        [b'.' | b'/', ..] => Some(&line[1..]),
        _ => None,
    }
}
