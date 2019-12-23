mod ircv3;
mod messaging;

use hexchat_plugin::{EAT_NONE, EventAttrs, InfoId, PluginHandle, Word, WordEol};
use ircv3::{Message, split};
use std::mem::replace;


pub struct Sponge {
    pub signature: Option<String>,
    pub value: Option<Message>,
}

impl Sponge {
    pub fn put(&mut self, _ph: &mut PluginHandle, line: &str) {
        let mut new: Message = split(line);
        self.signature = Some(new.get_signature());
        self.value = Some(new);
    }

    pub fn pop(&mut self, signature: &str) -> Option<Message> {
        match &self.value {
            None => None,  // If we have no Message, return None.
            Some(_msg) => {
                // If we have a Message...
                if self.signature.as_ref().unwrap_or(&"".to_string()) == signature {
                    // ...and the Signature matches, return and delete Value.
                    replace(&mut self.value, None)
                } else {
                    // Otherwise, keep the Message and return None.
                    None
                }
            }
        }
    }
}


pub static mut CURRENT: Sponge = Sponge {
    signature: None,
    value: None,
};


pub fn cb_print(
    _ph: &mut PluginHandle, _word: Word,
) -> hexchat_plugin::Eat {
    //  TODO:
    //  Make signature
    //  event = CURRENT.pop(signature)
    //  if event:
    //      re-emit Print with User Badges, etc.
    //      EAT_ALL

    EAT_NONE
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub fn cb_server(
    ph: &mut PluginHandle, _word: Word, _word_eol: WordEol, attr: EventAttrs,
) -> hexchat_plugin::Eat {
    match ph.get_info(&InfoId::Network) {
        None => EAT_NONE,
        Some(network) => {
            if &network != "Twitch" {
                EAT_NONE
            } else {
                let msg: Message = split(attr.tags);
                if &msg.command == "PRIVMSG" {
                    //  Do not interfere with PRIVMSGs.
                    EAT_NONE
                } else {
                    messaging::handle_event(ph, msg)
                }
            }
        }
    }
}
