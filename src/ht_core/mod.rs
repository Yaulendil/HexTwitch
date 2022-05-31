mod callbacks;
mod events;
mod output;
mod storage;

use std::{collections::HashSet, ops::Deref};
use chrono::{DateTime, Utc};
use hexchat::{
    ChannelRef,
    EatMode,
    get_channel_name,
    get_network_name,
    PrintEvent,
};

use crate::{irc::Message, NETWORK, prefs::*};
use output::{
    alert_basic,
    alert_error,
    BADGES_UNKNOWN,
    change_topic,
    CHANNELS,
    FAKE_MODE_NAME,
    print_with_irc,
    print_without_irc,
    TABCOLORS,
};
use storage::{Action, ignore_next_print_event, recover_message, store_message};


/// Trim a slice of arguments from Hexchat into something workable. The initial
///     slice likely has only a few arguments that are not empty, with the rest
///     being placeholders left over from when the slice was first constructed
///     in C.
#[inline]
fn arg_trim(args: &[String]) -> &[String] {
    match args.iter().position(String::is_empty) {
        Some(i) => &args[..i],
        None => args,
    }
}


fn this_is_twitch() -> bool {
    match get_network_name() {
        Some(network) => network == NETWORK,
        None => false,
    }
}


/// Reset the Color of a newly-focused Tab.
pub fn cb_focus(_channel: ChannelRef) -> EatMode {
    if this_is_twitch() {
        TABCOLORS.reset();
    }
    EatMode::None
}


/// Hide a Join Event if it is fake.
pub fn cb_join(word: &[String], _dt: DateTime<Utc>) -> EatMode {
    if this_is_twitch() {
        // let user = &word[0];
        // let channel = &word[1];
        //
        // if let Some(hannel) = channel.strip_prefix("#") {
        //     if user == hannel {
        //         output::fake_mode(channel, user, true);
        //     }
        // }

        if !word[2].contains("tmi.twitch.tv") {
            EatMode::All
        } else {
            EatMode::None
        }
    } else {
        EatMode::None
    }
}


/// Hide a Mode Event if it is fake.
pub fn cb_mode(word: &[String], _dt: DateTime<Utc>) -> EatMode {
    if this_is_twitch() && word[0] == FAKE_MODE_NAME {
        EatMode::All
    } else {
        EatMode::None
    }
}


pub fn cb_notice(word: &[String], _dt: DateTime<Utc>) -> EatMode {
    if this_is_twitch() && callbacks::run(&word[0]) {
        EatMode::All
    } else {
        EatMode::None
    }
}


pub fn cb_print(etype: PrintEvent, word: &[String], _dt: DateTime<Utc>) -> EatMode {
    fn need_irc(etype: PrintEvent) -> bool {
        match etype {
            PrintEvent::YOUR_ACTION => false,
            PrintEvent::YOUR_MESSAGE => false,
            _ => true,
        }
    }

    if this_is_twitch() {
        let channel: String = get_channel_name();

        #[cfg(feature = "full-debug")]
        hexchat::print_plain(&format!(
            "{} < {}",
            etype.get_id(),
            crate::irc::Signature::new(
                Some(&channel),
                hexchat::strip_formatting(&word[0]),
            ),
        ));

        //  Determine what should be done with this event.
        match recover_message() {
            Action::Ignore => EatMode::None,
            Action::ProcPrint if need_irc(etype) => EatMode::None,
            Action::ProcPrint => print_without_irc(&channel, etype, word),
            Action::ProcIrc(msg) => print_with_irc(&channel, etype, word, msg),
        }
    } else {
        EatMode::None
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub fn cb_server(_word: &[String], _dt: DateTime<Utc>, raw: String) -> EatMode {
    if this_is_twitch() {
        let msg: Message = raw.parse().expect("Failed to parse IRC Message");

        #[cfg(feature = "full-debug")]
        hexchat::print_plain(&format!(
            "{} < {}",
            msg.command,
            msg.get_signature(),
        ));

        let opt_eat: Option<EatMode> = match msg.command.as_str() {
            //  Chat Messages.
            "PRIVMSG" => {
                store_message(msg);
                Some(EatMode::None)
            }
            "WHISPER" => events::whisper_recv(msg),

            //  Status updates.
            "HOSTTARGET" => events::hosttarget(msg),
            "RECONNECT" => events::reconnect(msg),
            "ROOMSTATE" => events::roomstate(msg),
            "USERNOTICE" => events::usernotice(msg),
            "USERSTATE" => events::userstate(msg),

            //  Moderator Actions.
            "CLEARMSG" => events::clearmsg(msg),
            "CLEARCHAT" => events::clearchat(msg),

            //  Suppress Hexchat spamming complaints that Twitch does not
            //      implement WHO and WHOIS Commands.
            //  TODO: If there is a way to prevent Hexchat from sending a WHO to
            //      every channel after connecting, that would be preferable to
            //      doing this.
            "421" if msg.trail == "Unknown command" =>
                match msg.args.get(1).map(String::as_str) {
                    Some("WHO") | Some("WHOIS") => Some(EatMode::Hexchat),
                    _ => Some(EatMode::None),
                }

            //  Other.
            _ => Some(EatMode::None),
        };

        match opt_eat {
            Some(mode) => mode,
            None => {
                //  Do not check for HTDEBUG setting here, because a failure in
                //      a handler, for a known type, is a bigger deal than just
                //      not having a handler for an unknown one. This needs to
                //      be noticed and fixed.
                alert_error(&format!("Handler for IRC Command failed: {raw}"));
                EatMode::None
            }
        }
    } else {
        EatMode::None
    }
}


pub fn cmd_pref_follow_hosts(_arg_full: &[String]) -> EatMode {
    match PREF_FOLLOW_HOSTS.toggle() {
        Ok(false) => alert_basic("Twitch hosts will NOT be followed to the target channel."),
        Ok(true) => alert_basic("Twitch hosts will now be followed to the target channel."),
        Err(..) => alert_error("FAILED to set Preference."),
    }

    EatMode::All
}


pub fn cmd_pref_announce(_arg_full: &[String]) -> EatMode {
    match PREF_ANNOUNCE.toggle() {
        Ok(false) => alert_basic("Announcements will NOT be shown with colored messages."),
        Ok(true) => alert_basic("Announcements will now be shown with colored messages."),
        Err(..) => alert_error("FAILED to set Preference."),
    }

    EatMode::All
}


pub fn cmd_pref_debug(_arg_full: &[String]) -> EatMode {
    match PREF_DEBUG.toggle() {
        Ok(false) => alert_basic("Extra debug info will NOT be shown."),
        Ok(true) => alert_basic("Extra debug info will now be shown."),
        Err(..) => alert_error("FAILED to set Preference."),
    }

    EatMode::All
}


pub fn cmd_pref_whisper_here(_arg_full: &[String]) -> EatMode {
    match PREF_WHISPERS.toggle() {
        Ok(false) => alert_basic("Twitch whispers will ONLY be shown in their own Tab"),
        Ok(true) => alert_basic("Twitch whispers will also show in the current Tab."),
        Err(..) => alert_error("FAILED to set Preference."),
    }

    EatMode::All
}


pub fn cmd_htinfo(_arg_full: &[String]) -> EatMode {
    hexchat::print_plain(crate::PLUGIN_INFO);
    EatMode::All
}


pub fn cmd_prediction(_arg_full: &[String]) -> EatMode {
    let predict = &CHANNELS.current().predictions;

    if predict.is_empty() {
        alert_basic("No active Prediction.");
    } else {
        alert_basic(format!(
            "Current Prediction ({}): {}",
            predict.mode().desc(),
            predict.deref(),
        ));
    }

    EatMode::All
}


pub fn cmd_reward(arg_full: &[String]) -> EatMode {
    match arg_trim(&arg_full[1..]) {
        [] => {
            //  Print the current Reward Names.
            alert_basic("REWARD EVENTS:");

            for reward in Reward::get_all() {
                alert_basic(&format!(
                    "{}: '{}'",
                    reward.id(),
                    reward.get().unwrap_or_default(),
                ));
            }
        }
        [uuid, content @ ..] => match uuid.parse::<Reward>() {
            Ok(reward) => match if content.is_empty() {
                //  Unset a Reward.
                reward.unset()
            } else {
                //  Set a Reward.
                reward.set(&content.join(" "))
            } {
                Ok(()) => alert_basic("Reward updated."),
                Err(()) => alert_error("FAILED to update Reward."),
            }
            Err(()) => alert_error("Invalid Reward ID."),
        }
    }

    EatMode::All
}


pub fn cmd_title(arg_full: &[String]) -> EatMode {
    let channel: String = arg_full[1].to_ascii_lowercase();
    let topic: String = arg_trim(&arg_full[2..]).join(" ");

    change_topic(&channel, &topic);

    EatMode::All
}


pub fn cmd_tjoin(arg_full: &[String]) -> EatMode {
    cmd!(
        "JOIN {}",
        arg_trim(&arg_full[1..]).join(" "),
    );

    EatMode::All
}


pub fn cmd_unk_badges(_arg_full: &[String]) -> EatMode {
    let unk: &HashSet<String> = &BADGES_UNKNOWN.get();

    if unk.is_empty() {
        alert_basic("No unknown Badges have been seen.");
    } else {
        alert_basic("The following Badges do not have associated icons:");

        let mut vec: Vec<&String> = unk.iter().collect();
        vec.sort_unstable();

        for s in vec {
            alert_basic(s);
        }
    }

    EatMode::All
}


pub fn cmd_whisper(arg_full: &[String]) -> EatMode {
    let arg: &[String] = arg_trim(arg_full);

    match arg {
        [] | [_] => {}
        _ if !this_is_twitch() => {}
        [_, name] => {
            //  Switch to Whisper Tab.
            cmd!("QUERY {}", name);
        }
        [_, name, words @ ..] => {
            //  Send through Whisper.
            cmd!("SAY .w {} {}", name, words.join(" "));
        }
    }

    EatMode::All
}
