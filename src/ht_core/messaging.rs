use hexchat_plugin::{Eat, EAT_NONE, PluginHandle};
use super::ircv3;


pub fn handle_event(ph: &mut PluginHandle, msg: ircv3::Message) -> Eat {
    EAT_NONE
}
