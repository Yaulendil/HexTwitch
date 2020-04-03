use hexchat::{
    ChannelRef,
    EatMode,
    get_channel,
    get_pref_string,
    print_event_to_channel,
    PrintEvent,
    send_command,
};

use super::ircv3::Message;
use super::output::{
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_ERR,
    EVENT_NORMAL,
    EVENT_REWARD,
};


const WHISPER_LEFT: &str = "==";
const WHISPER_RIGHT: &str = "==";


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
        if let Some(notif) = get_pref_string(&custom) {
            //  We know what it should be called.
            echo(EVENT_REWARD, &[
                &notif,
                &format!("{}:", msg.author.display_name()),
                &word[1],
            ], 2);
        } else {
            //  We do NOT know what it should be called. Use a generic "CUSTOM"
            //      label, and also print the ID.
            echo(EVENT_REWARD, &[
                "CUSTOM",
                &format!("({}) {}:", custom, msg.author.display_name()),
                &word[1],
            ], 2);
        }

        Some(EatMode::All)
    } else if "highlighted-message" == msg.get_tag("msg-id")? {
        echo(EVENT_ALERT, &[
            msg.author.display_name(),
            &word[1],
        ], 2);

        Some(EatMode::All)
    } else { None }
}


fn raid(msg: &Message) -> Option<EatMode> {
    echo(EVENT_NORMAL, &[format!(
        "A raid of {} arrives from #{}",
        msg.get_tag("msg-param-viewerCount")?,
        msg.get_tag("msg-param-displayName")?.to_lowercase(),
    )], 1);
    Some(EatMode::Hexchat)
}


fn special(msg: &Message, _stype: &str) -> Option<EatMode> {
    echo(
        EVENT_NORMAL,
        &[msg.get_tag("system-msg")?],
        1,
    );
    Some(EatMode::Hexchat)
}


fn subscription(msg: &Message, stype: &str) -> Option<EatMode> {
    match stype {
        "sub" | "resub" => {
            let mut line = format!("<{}> {}scribes", msg.get_tag("login")?, stype);

            if let Some(plan) = msg.get_tag("msg-param-sub-plan") {
                match &*plan {
                    "Prime" => { line.push_str(" with Twitch Prime") }
                    // "1000" => { line.push_str(" at Tier 1 ($5)") }
                    "2000" => { line.push_str(" at Tier 2 ($10)") }
                    "3000" => { line.push_str(" at Tier 3 ($25)") }
                    _ => {}
                };
            }

            if let Some(streak) = msg.get_tag("msg-param-streak-months") {
                if streak.parse().unwrap_or(0) > 1 {
                    line.push_str(&format!(" for ({}) months in a row", streak));
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if cumul.parse().unwrap_or(0) > 1 {
                    line.push_str(&format!(", with ({}) months in total", cumul));
                }
            }

            if &msg.trail != "" { line.push_str(&format!(": {}", msg.trail)) };

            echo(EVENT_ALERT, &["SUBSCRIPTION", &line], 2);
        }

        "subgift" => {
            let mut line = format!(
                "<{}> is gifted a subscription by <{}>",
                msg.get_tag("msg-param-recipient-user-name")?,
                msg.get_tag("login")?,
            );

            if let Some(streak) = msg.get_tag("msg-param-months") {
                if streak.parse().unwrap_or(0) > 1 {
                    line.push_str(&format!(" for ({}) months in a row", streak));
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if cumul.parse().unwrap_or(0) > 1 {
                    line.push_str(&format!(", with ({}) months in total", cumul));
                }
            }

            echo(EVENT_ALERT, &["SUBSCRIPTION", &line], 2);
        }
        "submysterygift" => {
            let num = msg.get_tag("msg-param-mass-gift-count")?;
            echo(EVENT_ALERT, &["SUBSCRIPTION", &format!(
                "<{}> gives out ({}) random gift subscription{}",
                msg.get_tag("login")?,
                num,
                if &num == "1" { "" } else { "s" },
            )], 2);
        }

        "giftpaidupgrade" => {
            echo(EVENT_ALERT, &["UPGRADE", &format!(
                "<{}> upgrades a gift subscription from <{}>",
                msg.get_tag("login")?,
                msg.get_tag("msg-param-sender-login")?,
            )], 2);
        }
        "primepaidupgrade" => {
            echo(EVENT_ALERT, &["UPGRADE", &format!(
                "<{}> upgrades a Twitch Prime subscription",
                msg.get_tag("login")?,
            )], 2);
        }

        "bitsbadgetier" => {
            echo(EVENT_ALERT, &["BITS BADGE", &format!(
                "<{}> earns a new tier of Bits Badge",
                msg.get_tag("login")?,
            )], 1);
        }

        _ => {
            echo(EVENT_NORMAL, &[format!(
                "Unknown SType '{}': {}",
                stype,
                msg.get_tag("system-msg").unwrap_or_else(|| msg.as_str())
            )], 1);
        }
    }
    Some(EatMode::Hexchat)
}


pub fn ensure_tab(name: &str) -> ChannelRef {
    let channel: ChannelRef;

    if let Some(check) = get_channel("Twitch", &name) {
        channel = check;
    } else {
        send_command(&format!("QUERY {}", &name));
        send_command(&format!("SETTAB {}{}{}", WHISPER_LEFT, &name, WHISPER_RIGHT));
        channel = get_channel("Twitch", &name)
            .expect("Failed to ensure Whisper Tab.");
    }

    channel
}


pub fn whisper_recv(msg: Message) -> Option<EatMode> {
    let user: &str = &msg.author.user;
    let channel: ChannelRef = ensure_tab(user);

    let etype: PrintEvent;
    let text: &str;

    if msg.trail.starts_with("/me ") {
        etype = PrintEvent::PRIVATE_ACTION_TO_DIALOG;
        text = &msg.trail[4..];
    } else {
        etype = PrintEvent::PRIVATE_MESSAGE_TO_DIALOG;
        text = &msg.trail;
    }

    print_event_to_channel(&channel, etype, &[user, text, ""]);
    Some(EatMode::All)
}


pub fn whisper_send(etype: PrintEvent, user: &str, word: &[String]) {
    //  "asdf qwert" normal -> exec SAY ".w asdf qwert"
    //  ".w asdf qwert" -> emit "asdf qwert" as private

    let channel: ChannelRef = ensure_tab(user);
    let mut text: &str = &word[1];

    if text.starts_with(".w ") {
        //  Normal Message, begins with ".w". The Whisper has been sent. Print
        //      the message in the Tab.
        text = text.splitn(3, " ").last().unwrap();

        let etype_dm: PrintEvent = match etype {
            PrintEvent::YOUR_ACTION => PrintEvent::PRIVATE_ACTION_TO_DIALOG,
            PrintEvent::YOUR_MESSAGE if text.starts_with("/me ") => {
                text = &text[4..];
                PrintEvent::PRIVATE_ACTION_TO_DIALOG
            }
            PrintEvent::YOUR_MESSAGE => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
            _ => PrintEvent::PRIVATE_MESSAGE_TO_DIALOG,
        };

        print_event_to_channel(&channel, etype_dm, &[
            &*word[0],
            text,
            &*word[2],
        ]);
    } else {
        //  Normal Message, does NOT begin with ".w". Need to send the Whisper.
        //      Execute SAY on the message with ".w" prepended.
        if etype == PrintEvent::YOUR_ACTION {
            send_command(&format!("SAY .w {} /me {}", &user, text));
        } else {
            send_command(&format!("SAY .w {} {}", &user, text));
        }
    }
}


pub fn usernotice(msg: Message) -> Option<EatMode> {
    let stype = msg.get_tag("msg-id")?;
    match stype.as_str() {
        "raid" => raid(&msg),
        "charity" | "rewardgift" | "ritual" => special(&msg, &stype),
        _ => subscription(&msg, &stype),
    }
}


pub fn hosttarget(target: &str) -> Option<EatMode> {
    if target != "-" {
        echo(
            EVENT_CHANNEL,
            &[format!("#{}", target), format!("https://www.twitch.tv/{}", target)],
            1,
        );
    }

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    echo(
        EVENT_ERR,
        &[format!("A message by <{}> is deleted: {}",
                  msg.get_tag("login")?, &msg.trail)],
        1,
    );
    Some(EatMode::Hexchat)
}


pub fn clearchat(msg: Message) -> Option<EatMode> {
    let mut text: String = if let Some(t) = msg.get_tag("ban-duration") {
        format!("{} is timed out for {}s.", &msg.trail, t)
    } else {
        format!("{} is banned permanently.", &msg.trail)
    };

    if let Some(reason) = msg.get_tag("ban-reason") {
        if &reason != "" {
            text.push_str(&format!(" Reason: {}", reason));
        }
    }

    echo(EVENT_ERR, &[text], 1);
    Some(EatMode::Hexchat)
}
