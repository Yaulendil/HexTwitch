use hexchat::{
    EatMode,
    get_channel_name,
    get_current_channel,
    get_network_name,
    print_event_to_channel,
    PrintEvent,
};
use super::ircv3::Message;
use super::printing::USERSTATE;


fn raid(msg: &Message) -> Option<EatMode> {
    print_event_to_channel(
        &get_current_channel(),
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
    print_event_to_channel(
        &get_current_channel(),
        PrintEvent::MOTD,
        &[msg.trail.replace("\\s", " ")],
    );
    Some(EatMode::Hexchat)
}


fn subscription(msg: &Message, stype: &str) -> Option<EatMode> {
    Some(EatMode::None)
}


pub fn whisper(msg: Message) -> Option<EatMode> {
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

    print_event_to_channel(
        &get_current_channel(),
        PrintEvent::CHANNEL_URL,
        &[&format!("#{}", target), &format!("https://www.twitch.tv/{}", target)],
    );

    Some(EatMode::Hexchat)
}


pub fn clearmsg(msg: Message) -> Option<EatMode> {
    Some(EatMode::None)
}


pub fn clearchat(msg: Message) -> Option<EatMode> {
    Some(EatMode::None)
}
