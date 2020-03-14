mod printing;
mod tabs;


use hexchat::{
    EatMode,
    get_pref_string,
    PrintEvent,
    send_command,
};

pub use printing::{
    Badges,
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_ERR,
    EVENT_NORMAL,
    EVENT_REWARD,
    USERSTATE,
};
pub use tabs::TABCOLORS;
use super::events;
use super::ircv3::Message;


/// Message comes from Server. IRC Representation available.
pub fn print_with_irc(channel: &str, etype: PrintEvent, word: &[String], msg: Message) -> EatMode {
    if let Some(tags) = &msg.tags {
        if let Some(bits) = tags.get("bits") {
            if let Ok(n) = bits.parse::<usize>() {
                events::cheer(&msg.author.display_name(), n);
            }
        }

        if let Some(custom) = tags.get("custom-reward-id") {
            //  This Message is a Custom Reward.
            if let Some(notif) = get_pref_string(custom) {
                //  We know what it should be called.
                echo(
                    EVENT_REWARD,
                    &[
                        &notif,
                        &format!("{}:", &msg.author.display_name()),
                        &word[1],
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
                        &*word[1],
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
                    msg.author.display_name(),
                    &*word[1],
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
            echo(etype, &[&*word[0], &*word[1], "_", &*badge_str], 2);

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
                &[&*word[0], &*word[1], "", &*badges.output],
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
pub fn print_without_irc(channel: &str, etype: PrintEvent, word: &[String]) -> EatMode {
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
