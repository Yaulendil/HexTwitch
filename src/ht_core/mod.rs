mod ircv3;

use hexchat_plugin::{EAT_NONE, EventAttrs, PluginHandle, Word, WordEol};
use ircv3::{Message, split};
use std::mem::replace;


pub struct Sponge {
    pub signature: Option<String>,
    pub value: Option<Message>,
}

impl Sponge {
    pub fn put(&mut self, _ph: &mut PluginHandle, line: String) {
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


pub fn cb_server(
    _ph: &mut PluginHandle, _word: Word, _word_eol: WordEol, attr: EventAttrs,
) -> hexchat_plugin::Eat {
//    let msg: Message = split(&attr.tags.as_str());
    //  TODO:
    //  if msg.command != "PRIVMSG":
    //      Check for Subscriptions, etc.

    EAT_NONE
}
