pub(super) mod prediction;
mod printing;
mod statics;
mod tabs;

use hexchat::{EatMode, PrintEvent, send_command};
use crate::irc::{Message, Signature};
use super::{events, mark_processed};
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
pub use statics::{BADGES_UNKNOWN, PREDICTIONS, TABCOLORS, USERSTATE};
pub use tabs::TabColor;


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
            mark_processed(msg.get_signature());
            echo(etype, &[
                &*word[0], &*word[1], &*word[2],
                &USERSTATE.get(&channel),
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

            mark_processed(msg.get_signature());
            echo(
                etype,
                &[&*word[0], &*word[1], &*word[2], badges.as_str()],
                if etype == PrintEvent::CHANNEL_MSG_HILIGHT
                    || etype == PrintEvent::CHANNEL_ACTION_HILIGHT
                { TabColor::Highlight } else { TabColor::Message },
            );

            badges.update_prediction(&channel);

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
    if word[1].starts_with(".w ") {
        //  User has executed `.w`. We must take the message typed, and forward
        //      it to the Whisper Handler.
        events::whisper_send_command(etype, &channel, word);

        EatMode::All
    } else if !channel.starts_with::<&[char]>(&['#', '&']) {
        //  User has spoken inside a Whisper Tab. We must take the message
        //      typed, and forward it to the Whisper Handler.
        events::whisper_send_channel(etype, &channel, word);

        EatMode::All
    } else {
        let author: &str = &word[0];

        //  FIXME: Currently, a `/ME` Command executed by the User is not given
        //      the User Badges, while it IS given Badges when received from the
        //      Server. This seems to be where that goes wrong, but it is not
        //      clear why.
        //  User has spoken in a normal Channel, but has not yet been given
        //      Badges. Add the Badges from the User State and re-emit.
        mark_processed(Signature::new(Some(channel), Ok(author)));
        echo(etype, &[
            author, &*word[1], &*word[2],
            &USERSTATE.get(&channel),
        ], TabColor::Message);

        EatMode::All
    }
}
