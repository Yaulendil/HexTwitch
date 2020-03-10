use hexchat::{EatMode, get_channel_name};

use super::ircv3::Message;
use super::printing::{echo, EVENT_ALERT, EVENT_CHANNEL, EVENT_ERR, EVENT_NORMAL, USERSTATE};


/// This Macro will declare a Macro which returns a String Literal. Meant as a
///     way to hack some sort of `const` functionality into `format!()` calls.
macro_rules! constant {
    ($name:ident, $output:literal) => {
        macro_rules! $name {
            () => ($output)
        }
    }
}
constant!(RAID, "A raid of {} arrives from #{}");


fn raid(msg: &Message) -> Option<EatMode> {
    echo(
        EVENT_NORMAL,
        &[format!(
            RAID!(),
            msg.get_tag("msg-param-viewerCount")?,
            msg.get_tag("msg-param-displayName")?.to_lowercase(),
        )],
    );
    Some(EatMode::Hexchat)
}


fn special(msg: &Message, _stype: &str) -> Option<EatMode> {
    echo(
        EVENT_NORMAL,
        &[msg.get_tag("system-msg")?],
    );
    Some(EatMode::Hexchat)
}


fn subscription(msg: &Message, stype: &str) -> Option<EatMode> {
    match stype {
        "sub" | "resub" => {
            let mut line = format!("<{}> {}scribes", msg.get_tag("login")?, stype);

            if let Some(plan) = msg.get_tag("msg-param-sub-plan") {
                if &plan == "Prime" { line.push_str(" with Twitch Prime") };
            }

            if let Some(streak) = msg.get_tag("msg-param-streak-months") {
                if &streak != "1" {
                    line.push_str(&format!(" for ({}) months in a row", streak));
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if &cumul != "1" {
                    line.push_str(&format!(", with ({}) months in total", cumul));
                }
            }

            if &msg.trail != "" { line.push_str(&format!(": {}", msg.trail)) };

            echo(EVENT_ALERT, &["SUBSCRIPTION", &line]);
        }

        "subgift" => {
            let mut line = format!(
                "<{}> is gifted a subscription by <{}>",
                msg.get_tag("msg-param-recipient-user-name")?,
                msg.get_tag("login")?,
            );

            if let Some(streak) = msg.get_tag("msg-param-months") {
                if &streak != "1" {
                    line.push_str(&format!(" for ({}) months in a row", streak));
                }
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                if &cumul != "1" {
                    line.push_str(&format!(", with ({}) months in total", cumul));
                }
            }

            echo(EVENT_ALERT, &["SUBSCRIPTION", &line]);
        }
        "submysterygift" => {
            let num = msg.get_tag("msg-param-mass-gift-count")?;
            echo(EVENT_ALERT, &["SUBSCRIPTION", &format!(
                "<{}> gives out ({}) random gift subscription{}",
                msg.get_tag("login")?,
                num,
                if &num == "1" { "" } else { "s" },
            )]);
        }

        "giftpaidupgrade" => {
            echo(EVENT_ALERT, &["UPGRADE", &format!(
                "<{}> upgrades a gift subscription from <{}>",
                msg.get_tag("login")?,
                msg.get_tag("msg-param-sender-login")?,
            )]);
        }
        "primepaidupgrade" => {
            echo(EVENT_ALERT, &["UPGRADE", &format!(
                "<{}> upgrades a Twitch Prime subscription",
                msg.get_tag("login")?,
            )]);
        }

        "bitsbadgetier" => {
            echo(EVENT_ALERT, &["BITS BADGE", &format!(
                "<{}> earns a new tier of Bits Badge",
                msg.get_tag("login")?,
            )]);
        }

        _ => {
            echo(EVENT_NORMAL, &[format!(
                "Unknown SType '{}': {}",
                stype,
                msg.get_tag("system-msg").unwrap_or_else(|| msg.as_str())
            )]);
        }
    }
    Some(EatMode::Hexchat)
}


pub fn whisper(msg: Message) -> Option<EatMode> {
    //  TODO
    Some(EatMode::None)
}


pub fn userstate(msg: Message) -> Option<EatMode> {
    USERSTATE.write().set(
        get_channel_name(),
        &msg.get_tag("badges").unwrap_or_else(String::new),
    );
    Some(EatMode::All)
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
        );
    }

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    echo(
        EVENT_ERR,
        &[format!("A message by <{}> was deleted: {}",
                  msg.get_tag("login")?, &msg.trail)],
    );
    Some(EatMode::Hexchat)
}


pub fn clearchat(msg: Message) -> Option<EatMode> {
    let mut text: String = if let Some(t) = msg.get_tag("ban-duration") {
        format!("{} was timed out for {}s.", &msg.trail, t)
    } else {
        format!("{} was banned permanently.", &msg.trail)
    };

    if let Some(reason) = msg.get_tag("ban-reason") {
        if &reason != "" {
            text.push_str(&format!(" Reason: {}", reason));
        }
    }

    echo(EVENT_ERR, &[text]);
    Some(EatMode::Hexchat)
}
