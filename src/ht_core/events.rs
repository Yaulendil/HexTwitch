use hexchat::{EatMode, get_channel_name};

use super::ircv3::Message;
use super::printing::{
    echo,
    EVENT_ALERT,
    EVENT_CHANNEL,
    EVENT_ERR,
    EVENT_NORMAL,
    EVENT_REWARD,
    USERSTATE,
};


// /// This Macro will declare a Macro which returns a String Literal. Meant as a
// ///     way to hack some sort of `const` functionality into `format!()` calls.
// macro_rules! constant {
//     ($name:ident, $output:literal) => {
//         macro_rules! $name {
//             () => ($output)
//         }
//     }
// }
// constant!(RAID, "A raid of {} arrives from #{}");


pub fn cheer(name: &str, number: usize) {
    if number > 0 {
        echo(
            EVENT_REWARD,
            &[
                "CHEER",
                &format!("{} cheers", name),
                &format!("{} bit{}", number, if number == 1 { "" } else { "s" }),
            ],
            1,
        );
    }
}


pub fn reward(id: &str) -> Option<&str> {
    //  TODO: Do not hardcode this.
    match id {
        "e7c7bd6b-daf0-47fc-8bcf-8912c18bb964" => Some("QUESTION"),  // Coestar
        "8d4bf0fb-6964-4c58-90ab-1898c4b12133" => Some("VINYL"),  // Coestar
        "4391ebde-5287-4a68-a4ca-a60487b7500c" => Some("QUESTION"),  // Millbee
        "4908519c-3084-48ba-ba3c-ff00ee9c5331" => Some("RATE THING"),  // Millbee
        "ae58f3f3-4c88-4f3d-bae0-580a5c8193ac" => Some("VOICE"),  // Millbee
        "d8f09468-6bb3-4625-9baf-6682c4ad4da5" => Some("SAY NICE"),  // Millbee
        _ => None,
    }
}


fn raid(msg: &Message) -> Option<EatMode> {
    echo(
        EVENT_NORMAL,
        &[format!(
            "A raid of {} arrives from #{}",
            msg.get_tag("msg-param-viewerCount")?,
            msg.get_tag("msg-param-displayName")?.to_lowercase(),
        )],
        1,
    );
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

            echo(EVENT_ALERT, &["SUBSCRIPTION", &line], 2);
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
            1,
        );
    }

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    echo(
        EVENT_ERR,
        &[format!("A message by <{}> was deleted: {}",
                  msg.get_tag("login")?, &msg.trail)],
        1,
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

    echo(EVENT_ERR, &[text], 1);
    Some(EatMode::Hexchat)
}
