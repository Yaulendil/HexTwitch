mod ircv3;

use hexchat_plugin::{EAT_NONE, EventAttrs, PluginHandle, Word, WordEol};
use ircv3::{Message, split};


pub struct Sponge<'a> {
    pub value: Option<Message<'a>>,
}

impl<'a> Sponge<'a> {
    pub fn put(&mut self, _ph: &mut PluginHandle, line: &'a str) {
        let new: Message = split(line);
        self.value = Some(new);
    }

    pub fn pop(&mut self, signature: &str) -> Option<Message> {
        match &self.value {
            None => None,  // If we have no Message, return None.
            Some(msg) => {
                // If we have a Message...
                if msg.prefix == signature {
                    // ...and the Signature matches, delete Value.
                    self.value = None;
                    // And then, return the Message.
                    Some(msg)  // FIXME
                } else {
                    // Otherwise, keep the Message and return None.
                    None
                }
            }
        }
    }
}


pub static mut CURRENT: Sponge = Sponge {
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
