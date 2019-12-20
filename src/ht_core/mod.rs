mod ircv3;

use hexchat_plugin::{EventAttrs, PluginHandle, Word, WordEol};
use ircv3::{Message, split};


pub fn cb_server(
    ph: &mut PluginHandle, _word: Word, _word_eol: WordEol, attr: EventAttrs,
) -> hexchat_plugin::Eat {
    ph.print("RECEIVING");
    ph.print(attr.tags);  // DEBUG

    let msg: Message = split(attr.tags);

    hexchat_plugin::EAT_NONE
}
