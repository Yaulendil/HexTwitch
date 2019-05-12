mod messaging;
mod ircv3;

use ircv3::split;
use hexchat_plugin::{EventAttrs, PluginHandle, Word, WordEol};

pub fn cb_server(_ph: &mut PluginHandle, _word: Word, _word_eol: WordEol, _attr: EventAttrs) {}
