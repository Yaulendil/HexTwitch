use hexchat_plugin::{Eat, EAT_HEXCHAT, EAT_NONE, PluginHandle};
use super::ircv3::Message;


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


pub fn userstate(ph: &mut PluginHandle, msg: Message) -> Eat {
    EAT_NONE
}


pub fn usernotice(ph: &mut PluginHandle, msg: Message) -> Eat {
    match msg.tags.get("msg-id") {
        None => EAT_NONE,
        Some(_st) => {
            let stype: &str = _st.as_str();
            match stype {
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
