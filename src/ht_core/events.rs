use std::{collections::HashMap, fmt::Write};
use hexchat::{
    ChannelRef,
    EatMode,
    get_channel,
    get_channel_name,
    get_current_channel,
    get_focused_channel,
    print_event_to_channel,
    print_plain,
    PrintEvent,
};
use crate::{irc::{Message, split_at_char}, NETWORK, prefs::*};
use super::output::{
    alert_basic,
    alert_error,
    alert_subscription,
    alert_sub_upgrade,
    CHANNELS,
    channels::*,
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_REWARD,
    print_announcement,
    TabColor,
    USERSTATE,
};


/// Invocation of a self-action command.
const ME: &str = "/me ";
/// Length of the invocation of a self-action command. Defined by Constant to
///     avoid using a "magic number" to slice off the prefix.
const ME_LEN: usize = ME.len();


pub fn cheer(name: &str, number: usize) {
    if number > 0 {
        echo(EVENT_REWARD, &[
            "CHEER",
            &format!("{} cheers", name),
            &format!("{} bit{}", number, if number == 1 { "" } else { "s" }),
        ], TabColor::Event)
    }
}


pub fn reward(word: &[String], msg: &Message) -> Option<EatMode> {
    const REWARD_UNKNOWN: &str = "CUSTOM";

    if let Some(id) = msg.get_tag("custom-reward-id") {
        //  This Message is a Custom Reward.
        let reward_owned: String;
        let reward_name: &str;
        let author_name: String;

        match id.parse::<Reward>() {
            //  [CUSTOM] (No ID) username: message
            Err(()) if id.is_empty() => {
                //  Found an instance of this class of message. It is not a
                //      highlight. It is not a reward. It is just a regular
                //      standard message. Twitch. Why.

                if PREF_DEBUG.is(&true) {
                    msg.debug();
                }

                // reward_name = REWARD_UNKNOWN;
                // author_name = format!("(No ID) {}:", msg.author());
                return None;
            }

            //  [CUSTOM] (1334121037) username: message
            Err(()) => {
                reward_name = REWARD_UNKNOWN;
                author_name = format!("({}) {}:", id, msg.author());
            }

            Ok(reward) => match reward.get() {
                //  [Reward] username: message
                Some(title_pref) => {
                    reward_owned = title_pref;
                    reward_name = &reward_owned;
                    author_name = format!("{}:", msg.author());
                }

                //  [CUSTOM] (00000000-0000-0000-0000-000000000000) username: message
                None => {
                    reward_name = REWARD_UNKNOWN;
                    author_name = format!("({}) {}:", id, msg.author());
                }

                /*//  [00000000-0000-0000-0000-000000000000] username: message
                None => {
                    reward_owned = id;
                    reward_name = &reward_owned;
                    id_author = format!("{}:", msg.author());
                }*/
            }
        }

        echo(EVENT_REWARD, &[
            reward_name,
            &author_name,
            &word[1],
        ], TabColor::Message);

        Some(EatMode::All)
    } else if "highlighted-message" == msg.get_tag("msg-id")? {
        echo(EVENT_ALERT, &[
            msg.author(),
            &word[1],
        ], TabColor::Message);

        Some(EatMode::All)
    } else { None }
}


pub fn reconnect(msg: Message) -> Option<EatMode> {
    echo(PrintEvent::SERVER_NOTICE, &[
        "IRC Service is about to restart.",
        msg.prefix.server().unwrap_or(NETWORK),
    ], TabColor::None);

    Some(EatMode::All)
}


pub fn roomstate(msg: Message) -> Option<EatMode> {
    let tags: &HashMap<String, String> = msg.tags.as_ref()?;
    let join: bool = tags.len() > 2;
    // let debug: bool = PREF_DEBUG.is(&true);

    let roomstate: &mut RoomState = &mut CHANNELS.current().roomstate;
    let mut tags_vec: Vec<(&String, &String)> = tags.iter().collect();
    tags_vec.sort_unstable();

    for (k, v) in tags_vec {
        //  TODO: Move match into RoomState method, but retain logic here:
        //      let update = roomstate.update(k, v);
        //      let report: bool = {...};
        //      if report {
        //          roomstate.report_update(update);
        //      }
        match roomstate.update(k, v) {
            //  On join, disabled: Do NOT print.
            //  On join, enabled: Print.
            //  Not on join, disabled: Print.
            //  Not on join, enabled: Print.
            // let print: bool = match (join, *val) {
            //     (true, false) => false,
            //     (true, true) => true,
            //     (false, true) => true,
            //     (false, false) => true,
            // };
            // let print: bool = !(join && !*val);
            // let print: bool = !join || *val;
            //
            //  Only print on join when enabled:
            // let print: bool = join && *val;
            Ok(StateChange::Emotes(val)) => if join && *val {
                roomstate.report_emotes();
            }
            Ok(StateChange::Unique(val)) => if join && *val {
                roomstate.report_unique();
            }
            Ok(StateChange::Subscribers(val)) => if join && *val {
                roomstate.report_subscribers();
            }
            Ok(StateChange::Followers(val)) if join => match *val {
                FollowMode::Off => {} // No print on Join update.
                _ => roomstate.report_followers(),
            }
            Ok(StateChange::Slow(val)) if join => match *val {
                None => {} // No print on Join update.
                _ => roomstate.report_slow(),
            }
            // Ok(StateChange::Rituals(val)) if debug => match *val {
            //     None if join => {} // No print on Join update.
            //     _ => roomstate.report_rituals(),
            // }
            // Ok(StateChange::RoomId(val)) if debug => match *val {
            //     None if join => {} // No print on Join update.
            //     _ => roomstate.report_id(),
            // }
            Ok(..) => {}
            Err(_) => print_plain(&format!(
                "Unknown RoomState key {k:?} has value {v:?}.",
            )),
        }
    }

    Some(EatMode::Hexchat)
}


pub fn usernotice(msg: Message) -> Option<EatMode> {
    let stype: String = msg.get_tag("msg-id")?;

    match stype.as_str() {
        "announcement" => {
            return print_announcement(msg);
        }
        "raid" => {
            alert_basic(&format!(
                "A raid of {} arrives from #{}",
                msg.get_tag("msg-param-viewerCount")?,
                msg.get_tag("msg-param-displayName")?.to_lowercase(),
            ));
        }
        "charity" | "rewardgift" | "ritual" => {
            alert_basic(&msg.get_tag("system-msg")?);
        }

        "bitsbadgetier" => {
            let mut notif: String = match msg
                .get_tag("msg-param-threshold")
                .and_then(|t| t.parse::<usize>().ok())
            {
                Some(bits) => format!(
                    "<{}> earns a new tier of Bits Badge for cheering {} Bits \
                        (${:.2}) total",
                    msg.get_tag("login")?, bits,
                    bits as f64 * 0.01,
                ),
                None => format!(
                    "<{}> earns a new tier of Bits Badge",
                    msg.get_tag("login")?,
                ),
            };

            if !msg.trail.is_empty() { write!(&mut notif, ": {}", msg.trail).ok()?; }

            echo(EVENT_ALERT, &["BADGE", &notif], TabColor::Event);
        }

        "unraid" => alert_basic("A raid is canceled"),

        "sub" | "resub" => {
            // Maximum possible usage should be 362 bytes; 384=256+128
            let mut line: String = String::with_capacity(384);
            write!(&mut line, "<{}> {}scribes", msg.get_tag("login")?, stype).ok()?;

            if let Some(plan) = msg.get_tag("msg-param-sub-plan") {
                match plan.as_str() {
                    "Prime" => line.push_str(" with Prime"),
                    "1000" => line.push_str(" at Tier 1 ($5)"),
                    "2000" => line.push_str(" at Tier 2 ($10)"),
                    "3000" => line.push_str(" at Tier 3 ($25)"),
                    plan => write!(&mut line, " with plan {:?}", plan).ok()?,
                };
            }

            if let Some(streak) = msg.get_tag("msg-param-streak-months") {
                if streak.parse().unwrap_or(0) > 1 {
                    write!(&mut line, " for ({}) months in a row", streak).ok()?;
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if cumul.parse().unwrap_or(0) > 1 {
                    write!(&mut line, ", with ({}) months in total", cumul).ok()?;
                }
            }

            if !msg.trail.is_empty() { write!(&mut line, ": {}", msg.trail).ok()?; }

            alert_subscription(&line);
        }

        "extendsub" => {
            // Maximum possible usage should be 384 bytes.
            let mut line: String = String::with_capacity(384);
            write!(&mut line, "<{}> extends a sub", msg.get_tag("login")?).ok()?;

            if let Some(plan) = msg.get_tag("msg-param-sub-plan") {
                match plan.as_str() {
                    "Prime" => line.push_str(" with Prime"),
                    "1000" => { /*line.push_str(" at Tier 1 ($5)")*/ }
                    "2000" => line.push_str(" at Tier 2 ($10)"),
                    "3000" => line.push_str(" at Tier 3 ($25)"),
                    plan => write!(&mut line, " with plan {:?}", plan).ok()?,
                };
            }

            if let Some(streak) = msg.get_tag("msg-param-streak-months") {
                if streak.parse().unwrap_or(0) > 1 {
                    write!(&mut line, " for ({}) months in a row", streak).ok()?;
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if cumul.parse().unwrap_or(0) > 1 {
                    write!(&mut line, ", with ({}) months in total", cumul).ok()?;
                }
            }

            if let Some(month) = msg.get_tag("msg-param-sub-benefit-end-month") {
                write!(&mut line, ", through {}", match month.as_str() {
                    "1" => "January",
                    "2" => "February",
                    "3" => "March",
                    "4" => "April",
                    "5" => "May",
                    "6" => "June",
                    "7" => "July",
                    "8" => "August",
                    "9" => "September",
                    "10" => "October",
                    "11" => "November",
                    "12" => "December",
                    other => other,
                }).ok()?;
            }

            if !msg.trail.is_empty() { write!(&mut line, ": {}", msg.trail).ok()?; }

            alert_subscription(&line);
        }

        "subgift" => {
            let mut line: String = String::with_capacity(152);
            write!(
                &mut line,
                "<{}> is gifted a subscription by <{}>",
                msg.get_tag("msg-param-recipient-user-name")?,
                msg.get_tag("login")?,
            ).ok()?;

            if let Some(gifts) = msg.get_tag("msg-param-sender-count") {
                if gifts.parse().unwrap_or(0) > 0 {
                    write!(&mut line, " (Gifts: {})", gifts).ok()?;
                }
            }

            if let Some(streak) = msg.get_tag("msg-param-months") {
                if streak.parse().unwrap_or(0) > 1 {
                    write!(&mut line, " for ({}) months in a row", streak).ok()?;
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if cumul.parse().unwrap_or(0) > 1 {
                    write!(&mut line, ", with ({}) months in total", cumul).ok()?;
                }
            }

            alert_subscription(&line);
        }
        "submysterygift" => {
            let mut line: String = String::with_capacity(128);
            let num: String = msg.get_tag("msg-param-mass-gift-count")?;

            write!(
                &mut line,
                "<{}> gives out ({}) random gift subscription{}",
                msg.get_tag("login")?, num,
                if &num == "1" { "" } else { "s" },
            ).ok()?;

            if let Some(gifts) = msg.get_tag("msg-param-sender-count") {
                if gifts.parse().unwrap_or(0) > 0 {
                    write!(&mut line, " (Total: {})", gifts).ok()?;
                }
            }

            alert_subscription(&line);
        }
        "standardpayforward" => match msg.get_tag("msg-param-prior-gifter-user-name") {
            Some(prior) => alert_basic(&format!(
                "<{}> pays forward a gift subscription from <{}> to <{}>",
                msg.get_tag("login")?,
                prior,
                msg.get_tag("msg-param-recipient-user-name")?,
            )),
            None => alert_basic(&format!(
                "<{}> pays forward an anonymous gift subscription to <{}>",
                msg.get_tag("login")?,
                msg.get_tag("msg-param-recipient-user-name")?,
            )),
        }
        "communitypayforward" => match msg.get_tag("msg-param-prior-gifter-user-name") {
            Some(prior) => alert_basic(&format!(
                "<{}> pays forward a gift subscription from <{}> to the community",
                msg.get_tag("login")?,
                prior,
            )),
            None => alert_basic(&format!(
                "<{}> pays forward an anonymous gift subscription to the community",
                msg.get_tag("login")?,
            )),
        }

        "giftpaidupgrade" => alert_sub_upgrade(&format!(
            "<{}> upgrades a gift subscription from <{}>",
            msg.get_tag("login")?,
            msg.get_tag("msg-param-sender-login")?,
        )),
        "anongiftpaidupgrade" => alert_sub_upgrade(&format!(
            "<{}> upgrades an anonymous gift subscription",
            msg.get_tag("login")?,
        )),
        "primepaidupgrade" => alert_sub_upgrade(&format!(
            "<{}> upgrades a Prime subscription",
            msg.get_tag("login")?,
        )),

        _ => {
            if PREF_DEBUG.is(&true) {
                alert_error(&format!(
                    "Unknown UserNotice ID {:?}: {}",
                    stype, msg,
                ));
            }

            if let Some(sysmsg) = msg.get_tag("system-msg") {
                echo(EVENT_ALERT, &["UNKNOWN", &sysmsg], TabColor::Event);
            }
        }
    }

    Some(EatMode::Hexchat)
}


const fn badge_phrase(replacing: bool, empty: bool) -> &'static str {
    const INITIAL: bool = false;
    const REPLACE: bool = true;
    const SOME: bool = false;
    const NONE: bool = true;

    match (replacing, empty) {
        (INITIAL, NONE) => "No Badges received.",
        (REPLACE, NONE) => "Badges cleared.",

        (INITIAL, SOME) => "Badges received:",
        (REPLACE, SOME) => "New Badges received:",
    }
}


pub fn userstate(msg: Message) -> Option<EatMode> {
    /// The title put in brackets at the start of a state update message.
    const HEADER: &'static str = "BADGES";

    let channel: String = get_channel_name();
    if !channel.starts_with::<&[char]>(&['#', '&']) {
        //  If the channel is neither a #channel nor a &channel, return
        //      immediately. This is likely a whisper, which has no badges.
        return Some(EatMode::All);
    }

    let replacing: bool = USERSTATE.has(&channel);

    if let Some(badges) = USERSTATE.set(
        channel.clone(),
        msg.get_tag("badges").unwrap_or_default(),
        msg.get_tag("badge-info").unwrap_or_default(),
    ) {
        let empty = badges.is_empty();
        let phrase = badge_phrase(replacing, empty);

        if empty {
            echo(EVENT_REWARD, &[HEADER, phrase, ""], TabColor::None);
        } else {
            echo(EVENT_REWARD, &[
                HEADER,
                phrase,
                badges.as_str(),
            ], TabColor::None);

            badges.update_prediction(&channel);
        }
    }

    Some(EatMode::All)
}


/// Ensure Tab: Given a Channel Name, try to find it in the Twitch Network. If
///     it is not found, run the Hexchat Command to open it. Then, try to find
///     it again.
///
/// Input: `&str`
/// Return: `ChannelRef`
/// Panics: If the Channel is not found after `QUERY` is executed.
pub fn ensure_tab(name: &str) -> ChannelRef {
    get_channel(NETWORK, &name).unwrap_or_else(|| {
        cmd!("QUERY {}", &name);
        get_channel(NETWORK, &name).expect("Failed to ensure Whisper Tab.")
    })
}


/// Receive an IRC Message as a Twitch Whisper. The Message will be edited
///     somewhat, so that HexChat parses it in the right way.
///
/// NOTE: If this Function is given a Message whose Command field value is less
///     than 7 bytes, that `String` will reallocate.
///
/// Input: `Message`
/// Return: `Option<EatMode>`
pub fn whisper_recv(mut msg: Message) -> Option<EatMode> {
    let user: &str = msg.prefix.name();

    #[cfg(feature = "nightly")]
    //  Swap out fields of the Message to reshape it into one that HexChat can
    //      nicely handle for us.
    //  The Command is (fragilely) guaranteed to have previously been "WHISPER",
    //      which is the same length as "PRIVMSG", so we can probably avoid an
    //      allocation by overwriting it.
    //  TODO: Decide whether this is pointless. It feels pointless.
    "PRIVMSG".clone_into(&mut msg.command);

    #[cfg(not(feature = "nightly"))] {
        msg.command = String::from("PRIVMSG");
    }

    //  Change the first Argument to be the name of the author.
    msg.args[0] = user.to_owned();

    //  Action Messages have a different format than simply a `/me` command. For
    //      example, the command "/me does something" would have to be changed
    //      to "\x01ACTION does something\x01".
    if msg.trail.starts_with(ME) {
        let text: &str = &msg.trail[ME_LEN..]; // Slice off the `/me `.

        //  If the Whisper Tab is not focused, also post it here.
        if PREF_WHISPERS.is(&true) && get_channel_name() != user {
            echo(PrintEvent::PRIVATE_ACTION, &[user, text], TabColor::Message);
        }

        //  Format the sliced text into an Action Message and replace the Trail.
        msg.trail = format!("\x01ACTION {}\x01", &text);
    } else {
        //  If the Whisper Tab is not focused, also post it here.
        if PREF_WHISPERS.is(&true) && get_channel_name() != user {
            echo(
                PrintEvent::PRIVATE_MESSAGE,
                &[user, &msg.trail],
                TabColor::Message,
            );
        }
    }

    cmd!("RECV {}", msg);
    Some(EatMode::All)
}


pub fn whisper_send_channel(etype: PrintEvent, channel: &str, word: &[String]) {
    //  Normal Message, does NOT begin with ".w". Need to send the Whisper.
    //      Execute SAY on the message with ".w" prepended.
    if etype == PrintEvent::YOUR_ACTION {
        cmd!("SAY .w {} /me {}", channel, word[1]);
    } else {
        cmd!("SAY .w {} {}", channel, word[1]);
    }
}


pub fn whisper_send_command(etype: PrintEvent, channel: &str, word: &[String]) {
    //  Normal Message, begins with ".w". The Whisper has been sent. Print the
    //      message in the Tab.
    let (user, mut text) = split_at_char(word[1][3..].trim(), ' ');

    if !text.is_empty() {
        if user != channel {
            //  If the current tab is not the target tab, also print a line here
            //      confirming the message is sent.
            echo(PrintEvent::MESSAGE_SEND, &[user, text], TabColor::Message);
        }

        let etype_dm: PrintEvent = match etype {
            PrintEvent::YOUR_ACTION => PrintEvent::PRIVATE_ACTION_TO_DIALOG,
            PrintEvent::YOUR_MESSAGE if text.starts_with(ME) => {
                text = &text[ME_LEN..];
                PrintEvent::PRIVATE_ACTION_TO_DIALOG
            }
            PrintEvent::YOUR_MESSAGE => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
            _ => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
        };

        print_event_to_channel(&ensure_tab(user), etype_dm, &[
            word[0].as_str(), text, word[2].as_str(),
        ]);
    }
}


fn host_notif(viewers: &str) -> String {
    match viewers.parse::<usize>() {
        Ok(1) => String::from("Channel is hosted, with 1 viewer, by"),
        Ok(v) => format!("Channel is hosted, with {} viewers, by", v),
        _ => String::from("Channel is hosted by"),
    }
}


pub fn hosttarget(msg: Message) -> Option<EatMode> {
    let (target, viewers) = split_at_char(&msg.trail, ' ');

    if !target.is_empty() && target != "-" {
        let hashtarg: String = format!("#{}", target);

        //  Check whether we should try to follow the host.
        if PREF_FOLLOW_HOSTS.is(&true) {
            let current = get_current_channel();
            let focused = get_focused_channel();

            //  Check whether the source channel is currently focused.
            if Some(current) == focused {
                //  Join the target channel, if necessary.
                if get_channel(NETWORK, &hashtarg).is_none() {
                    cmd!("JOIN {}", hashtarg);
                }

                //  Focus the target channel.
                cmd!("DOAT {}/{} GUI focus", hashtarg, NETWORK);
            }
        }

        echo(EVENT_CHANNEL, &[
            &hashtarg, &format!("https://twitch.tv/{}", target),
        ], TabColor::Event);

        if let Some(channel) = get_channel(NETWORK, &hashtarg) {
            print_event_to_channel(&channel, EVENT_REWARD, &[
                "HOST",
                &host_notif(viewers),
                &msg.args[0],
            ]);
        }
    }

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    alert_error(&format!(
        "A message by <{}> is deleted: {}",
        msg.get_tag("login")?, &msg.trail,
    ));
    Some(EatMode::Hexchat)
}


pub fn clearchat(msg: Message) -> Option<EatMode> {
    if msg.trail.is_empty() {
        alert_error("Chat history has been cleared.");
    } else {
        let mut text: String = String::with_capacity(128);

        match msg.get_tag("ban-duration") {
            Some(t) => write!(&mut text, "{} is timed out for {}s", &msg.trail, t).ok()?,
            None => write!(&mut text, "{} is banned permanently", &msg.trail).ok()?,
        };

        if let Some(reason) = msg.get_tag("ban-reason") {
            if !reason.is_empty() {
                write!(&mut text, ". Reason: {}", reason).ok()?;
            }
        }

        alert_error(&text);
    }

    Some(EatMode::Hexchat)
}
