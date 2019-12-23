use hexchat_plugin::{Eat, EAT_ALL, EAT_HEXCHAT, EAT_NONE, PluginHandle};
use super::ircv3::Message;
use super::printing::USERSTATE;


fn raid(ph: &mut PluginHandle, msg: &Message) -> Eat {
    EAT_NONE
}


fn special(ph: &mut PluginHandle, msg: &Message, stype: &str) -> Eat {
    EAT_NONE
}


fn subscription(ph: &mut PluginHandle, msg: &Message, stype: &str) -> Eat {
    EAT_NONE
}


pub fn whisper(ph: &mut PluginHandle, msg: Message) -> Eat {
    EAT_NONE
}


pub unsafe fn userstate(ph: &mut PluginHandle, msg: Message) -> Eat {
    USERSTATE.set(
        format!("{}:{}",
                ph.get_info(&hexchat_plugin::InfoId::Network).expect("Network not found"),
                ph.get_info(&hexchat_plugin::InfoId::Channel).expect("Channel not found"),
        ),
        msg.tags.get("badges").unwrap(),
    );
    EAT_ALL
}


pub fn usernotice(ph: &mut PluginHandle, msg: Message) -> Eat {
    match msg.tags.get("msg-id") {
        None => EAT_NONE,
        Some(stype) => {
            match stype.as_str() {
                "raid" => raid(ph, &msg),
                "charity" | "rewardgift" | "ritual" => special(ph, &msg, stype),
                _ => subscription(ph, &msg, stype)
            }
        }
    }
}


pub fn hosttarget(ph: &mut PluginHandle, msg: Message) -> Eat {
    EAT_NONE
}


pub fn clearmsg(ph: &mut PluginHandle, msg: Message) -> Eat {
    EAT_NONE
}


pub fn clearchat(ph: &mut PluginHandle, msg: Message) -> Eat {
    EAT_NONE
}
