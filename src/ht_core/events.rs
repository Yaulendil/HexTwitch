use crate::NETWORK;
use hexchat::{
    ChannelRef,
    EatMode,
    get_channel,
    get_channel_name,
    get_pref_int,
    get_pref_string,
    print_event_to_channel,
    print_plain,
    PrintEvent,
    send_command,
};
use std::{
    collections::HashMap,
    fmt::Write,
};
use super::{
    irc::{Message, split_at_char},
    output::{
        alert_basic,
        alert_error,
        alert_subscription,
        alert_sub_upgrade,
        echo,
        EVENT_ALERT,
        EVENT_CHANNEL,
        EVENT_REWARD,
        USERSTATE,
    },
};


pub fn cheer(name: &str, number: usize) {
    if number > 0 {
        echo(EVENT_REWARD, &[
            "CHEER",
            &format!("{} cheers", name),
            &format!("{} bit{}", number, if number == 1 { "" } else { "s" }),
        ], 1)
    }
}


pub fn reward(word: &[String], msg: &Message) -> Option<EatMode> {
    if let Some(custom) = msg.get_tag("custom-reward-id") {
        //  This Message is a Custom Reward.
        match get_pref_string(&custom) {
            Some(notif) => echo(EVENT_REWARD, &[
                &notif,
                &format!("{}:", msg.author()),
                &word[1],
            ], 2),
            None => echo(EVENT_REWARD, &[
                "CUSTOM",
                &format!("({}) {}:", custom, msg.author()),
                &word[1],
            ], 2),
        }

        Some(EatMode::All)
    } else if "highlighted-message" == msg.get_tag("msg-id")? {
        echo(EVENT_ALERT, &[
            msg.author(),
            &word[1],
        ], 2);

        Some(EatMode::All)
    } else { None }
}


pub fn roomstate(msg: Message) -> Option<EatMode> {
    let tags: &HashMap<String, String> = msg.tags.as_ref()?;

    //  Only report this Message if it seems to be an On-Join update.
    if tags.len() > 2 {
        let mut tags_vec: Vec<(&String, &String)> = tags.iter().collect();
        tags_vec.sort_unstable();

        for (k, v) in tags_vec {
            match k.as_str() {
                "emote-only" => if v != "0" {
                    print_plain("Emotes Only mode enabled.");
                }
                "r9k" => if v != "0" {
                    print_plain("R9K mode enabled.");
                }
                "subs-only" => if v != "0" {
                    print_plain("Subscribers Only mode enabled.");
                }
                "slow" => {
                    let secs: isize = v.parse().unwrap_or(0);

                    if secs > 0 {
                        print_plain(&format!("Slow mode ({}s) enabled.", secs));
                    }
                }
                "followers-only" => {
                    // -1: No follow requirement.
                    //  0: Must follow to talk.
                    //  N: Must follow for N minutes before talking.
                    let mins: isize = v.parse().unwrap_or(-1);

                    if mins > 0 {
                        print_plain(&format!(
                            "{}-minute Followers Only mode enabled.",
                            mins,
                        ));
                    } else if mins == 0 {
                        print_plain("Followers Only mode enabled.");
                    }
                }
                "rituals" | "room-id" => { /* Ignore */ }
                key => print_plain(&format!(
                    "Unknown RoomState {:?} has value {:?}.",
                    key, v,
                )),
            }
        }
    }

    Some(EatMode::Hexchat)
}


pub fn usernotice(msg: Message) -> Option<EatMode> {
    let stype: String = msg.get_tag("msg-id")?;

    match stype.as_str() {
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
                    bits as f64 / 100.0,
                ),
                None => format!(
                    "<{}> earns a new tier of Bits Badge",
                    msg.get_tag("login")?,
                ),
            };

            if !msg.trail.is_empty() { write!(&mut notif, ": {}", msg.trail).ok()?; }

            echo(EVENT_ALERT, &["BADGE", &notif], 1);
        }

        "unraid" => alert_basic("A raid has been canceled"),

        "sub" | "resub" => {
            // Maximum possible usage should be 362 bytes; 384=256+128
            let mut line: String = String::with_capacity(384);
            write!(&mut line, "<{}> {}scribes", msg.get_tag("login")?, stype).ok()?;

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
            let num: String = msg.get_tag("msg-param-mass-gift-count")?;
            alert_subscription(&format!(
                "<{}> gives out ({}) random gift subscription{}",
                msg.get_tag("login")?, num,
                if &num == "1" { "" } else { "s" },
            ));
        }
        "standardpayforward" => {
            match msg.get_tag("msg-param-prior-gifter-user-name") {
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
            if get_pref_int("PREF_htdebug").unwrap_or(0) != 0 {
                alert_error(&format!(
                    "Unknown SType '{}': {}",
                    stype, msg,
                ));
            }

            if let Some(sysmsg) = msg.get_tag("system-msg") {
                echo(EVENT_ALERT, &["UNKNOWN", &sysmsg], 1);
            }
        }
    }

    Some(EatMode::Hexchat)
}


pub fn userstate(msg: Message) -> Option<EatMode> {
    let ch: String = get_channel_name();
    let mut state = USERSTATE.write();

    if state.set(
        &ch,
        &msg.get_tag("badges").unwrap_or_default(),
        &msg.get_tag("badge-info").unwrap_or_default(),
    ) {
        echo(EVENT_REWARD, &[
            "BADGES",
            "New Badges received:",
            state.get(&ch),
        ], 0);
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
        send_command(&format!("QUERY {}", &name));
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

    //  Swap out fields of the Message to reshape it into one that HexChat can
    //      nicely handle for us.
    //  The Command is (fragilely) guaranteed to have previously been "WHISPER",
    //      which is the same length as "PRIVMSG", so we can probably avoid an
    //      allocation by overwriting it.
    "PRIVMSG".clone_into(&mut msg.command);

    //  Change the first Argument to be the name of the author.
    msg.args[0] = user.to_owned();

    //  Action Messages have a different format than simply a `/me` command. For
    //      example, the command "/me does something" would have to be changed
    //      to "\x01ACTION does something\x01".
    if msg.trail.starts_with("/me ") {
        let text: &str = &msg.trail[4..]; // Slice off the `/me `.

        //  If the Whisper Tab is not focused, also post it here.
        if get_pref_int("PREF_whispers_in_current").unwrap_or(0) != 0
            && get_channel_name() != user
        {
            echo(PrintEvent::PRIVATE_ACTION, &[user, text], 2);
        }

        //  Format the sliced text into an Action Message and replace the Trail.
        msg.trail = format!("\x01ACTION {}\x01", &text);
    } else {
        //  If the Whisper Tab is not focused, also post it here.
        if get_pref_int("PREF_whispers_in_current").unwrap_or(0) != 0
            && get_channel_name() != user
        {
            echo(PrintEvent::PRIVATE_MESSAGE, &[user, &msg.trail], 2);
        }
    }

    send_command(&format!("RECV {}", msg));
    Some(EatMode::All)
}


pub fn whisper_send(etype: PrintEvent, channel: &str, word: &[String]) {
    //  "asdf qwert" normal -> exec SAY ".w asdf qwert"
    //  ".w asdf qwert" -> emit "asdf qwert" as private

    let text: &str = &word[1];

    if text.starts_with(".w ") {
        //  Normal Message, begins with ".w". The Whisper has been sent. Print
        //      the message in the Tab.
        let mut iter = text[3..].trim().splitn(2, " ");
        let user: &str = iter.next().unwrap();

        if let Some(mut text) = iter.next() {
            if user != channel {
                echo(PrintEvent::MESSAGE_SEND, &[&user, &text], 2);
            }

            let etype_dm: PrintEvent = match etype {
                PrintEvent::YOUR_ACTION => PrintEvent::PRIVATE_ACTION_TO_DIALOG,
                PrintEvent::YOUR_MESSAGE if text.starts_with("/me ") => {
                    text = &text[4..];
                    PrintEvent::PRIVATE_ACTION_TO_DIALOG
                }
                PrintEvent::YOUR_MESSAGE => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
                _ => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
            };

            print_event_to_channel(&ensure_tab(user), etype_dm, &[
                word[0].as_str(), text, word[2].as_str(),
            ]);
        }
    } else {
        //  Normal Message, does NOT begin with ".w". Need to send the Whisper.
        //      Execute SAY on the message with ".w" prepended.
        if etype == PrintEvent::YOUR_ACTION {
            send_command(&format!("SAY .w {} /me {}", &channel, text));
        } else {
            send_command(&format!("SAY .w {} {}", &channel, text));
        }
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

    if target != "-" {
        let hashtarg: String = format!("#{}", target);

        echo(EVENT_CHANNEL, &[
            &hashtarg, &format!("https://twitch.tv/{}", target),
        ], 1);

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
    Some(EatMode::Hexchat)
}
