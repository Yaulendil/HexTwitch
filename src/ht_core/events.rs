use hexchat::{
    EatMode,
    get_channel_name,
    get_network_name,
    PrintEvent,
};
use super::ircv3::Message;
use super::printing::{echo, USERSTATE};


fn raid(msg: &Message) -> Option<EatMode> {
    echo(
        PrintEvent::MOTD,
        &[format!(
            "{} sends {} raiders to this channel",
            msg.get_tag("msg-param-displayName")?,
            msg.get_tag("msg-param-viewerCount")?,
        )],
    );
    Some(EatMode::Hexchat)
}


fn special(msg: &Message, stype: &str) -> Option<EatMode> {
    echo(
        PrintEvent::MOTD,
        &[msg.trail.replace("\\s", " ")],
    );
    Some(EatMode::Hexchat)
}


fn subscription(msg: &Message, stype: &str) -> Option<EatMode> {
    //  TODO
    match stype {
        "sub" | "resub" => {
            let mut line = format!("<{}> {}scribes", msg.get_tag("login")?, stype);

            if let Some(plan) = msg.get_tag("msg-param-sub-plan") {
                if &plan == "Prime" { line.push_str(" with Twitch Prime") };
            }

            if let Some(streak) = msg.get_tag("msg-param-streak-months") {
                line.push_str(&format!(" for ({}) months in a row", streak));
            }

            if let Some(cumul) = msg.get_tag("msg-param-cumulative-months") {
                line.push_str(&format!(", with ({}) months in total", cumul));
            }

            if &msg.trail != "" { line.push_str(&format!(": {}", msg.trail)) };

            echo(PrintEvent::WHOIS_SERVER_LINE, &["SUBSCRIPTION", line]);
        }

        "subgift" => {}
        "submysterygift" => {}

        "giftpaidupgrade" => {}
        "primepaidupgrade" => {}

        "bitsbadgetier" => {}

        _ => {
            echo(PrintEvent::MOTD, &[format!(
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
        format!(
            "{}:{}",
            get_network_name().expect("Network not found"),
            get_channel_name(),
        ),
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


pub fn hosttarget(msg: Message) -> Option<EatMode> {
    let target = msg.trail.to_lowercase();

    echo(
        PrintEvent::CHANNEL_URL,
        &[format!("#{}", target), format!("https://www.twitch.tv/{}", target)],
    );

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    echo(
        PrintEvent::SERVER_ERROR,
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

    echo(PrintEvent::SERVER_ERROR, &[text]);
    Some(EatMode::Hexchat)
}
