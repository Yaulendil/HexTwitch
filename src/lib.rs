/*
 * Core package for the HexTwitch Rust Plugin.
 *
 * Purely experimental.
 */

#[macro_use]
extern crate hexchat_plugin;

mod ht_core;

use hexchat_plugin::{CommandHookHandle, Plugin, PluginHandle, ServerHookHandle};
use std::sync::Mutex;

const NAME: &str = env!("CARGO_PKG_NAME");
const DESC: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
struct HexTwitch {
    cmd: Mutex<Option<CommandHookHandle>>,
    servmsg: Mutex<Option<ServerHookHandle>>,
}

impl Plugin for HexTwitch {
    fn init(&self, ph: &mut PluginHandle, _arg: Option<&str>) -> bool {
        ph.register(NAME, DESC, VERSION);
        *self.cmd.lock().unwrap() = Some(ph.hook_command(
            "asdf",
            |ph, _arg, _arg_eol| {
                ph.print("qwert");
                hexchat_plugin::EAT_ALL
            },
            hexchat_plugin::PRI_NORM,
            Some("prints 'qwert'"),
        ));
        // Copied and modified the above for this next section. Dont understand it. At all.
        *self.servmsg.lock().unwrap() = Some(ph.hook_server_attrs(
            "RAW_LINE",
            |ph, _word, _word_eol, _attr| {
                ht_core::cb_server(ph, _word, _word_eol, _attr);
                hexchat_plugin::EAT_ALL
            },
            hexchat_plugin::PRI_NORM,
        ));
        true
    }
}

hexchat_plugin!(HexTwitch);

// This is from the example code. I dont even know where to start.
//use hexchat_plugin::{PluginHandle, ServerHookHandle};
fn register_server_hooks(ph: &mut PluginHandle) -> Vec<ServerHookHandle> {
    vec![
        ph.hook_server("PRIVMSG", |ph, word, word_eol| {
            if word.len() > 0 && word[0].starts_with('@') {
                ph.print("We have message tags!?");
            }
            hexchat_plugin::EAT_NONE
        }, hexchat_plugin::PRI_NORM),
    ]
}
