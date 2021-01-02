pub(super) mod prediction;
mod printing;
mod tabs;

use hexchat::{EatMode, PrintEvent, send_command};
pub use printing::{
    alert_basic,
    alert_error,
    alert_subscription,
    alert_sub_upgrade,
    badge_parse,
    Badges,
    BADGES_UNK,
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_ERR,
    EVENT_NORMAL,
    EVENT_REWARD,
    USERSTATE,
};
use super::{events, irc::Message};
pub use tabs::TABCOLORS;


/// Message comes from Server. IRC Representation available.
pub fn print_with_irc(
    channel: &str,
    etype: PrintEvent,
    word: &[String],
    msg: Message,
) -> EatMode {
    if msg.has_tags() {
        if let Some(bits) = msg.get_tag("bits") {
            if let Ok(n) = bits.parse::<usize>() {
                events::cheer(msg.author(), n);
            }
        }

        if let Some(eat) = events::reward(word, &msg) { return eat; }
    }

    match etype {
        PrintEvent::YOUR_MESSAGE
        | PrintEvent::YOUR_ACTION
        => {
            echo(etype, &[
                &*word[0], &*word[1], "_",
                USERSTATE.read().get(&channel),
            ], 2);

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

            echo(
                etype,
                &[&*word[0], &*word[1], "", &badges],
                if etype == PrintEvent::CHANNEL_MSG_HILIGHT
                    || etype == PrintEvent::CHANNEL_ACTION_HILIGHT
                { 3 } else { 2 },
            );

            badges.update_prediction();

            if msg.get_tag("anonymous-cheerer").is_none() {
                send_command(&format!(
                    "RECV :{0}!twitch.tv/{0} JOIN {1}",
                    msg.author(), channel,
                ));
            }

            EatMode::All
        }
        _ => EatMode::None,
    }
}


/// No IRC Representation available for Message.
pub fn print_without_irc(channel: &str, etype: PrintEvent, word: &[String]) -> EatMode {
    if word[1].starts_with(".w ") || !channel.starts_with::<&[char]>(&['#', '&']) {
        //  User has spoken inside a Whisper Tab, or executed `.w` elsewhere.
        //      We must take the message typed, and forward it to the Whisper
        //      Handler.
        events::whisper_send(etype, &channel, word);

        EatMode::All
    } else if word[2].is_empty() {
        //  FIXME: Currently, a `/ME` Command executed by the User is not given
        //      the User Badges, while it IS given Badges when received from the
        //      Server. This seems to be where that goes wrong, but it is not
        //      clear why.
        //  User has spoken in a normal Channel, but has no Badges.
        //      Add the Badges from the User State and re-emit.
        echo(etype, &[
            &*word[0], &*word[1], "_",
            USERSTATE.read().get(&channel),
        ], 2);

        EatMode::All
    } else {
        //  This is a re-emit. Do nothing.
        EatMode::None
    }
}
