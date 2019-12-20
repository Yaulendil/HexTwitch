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

        // Copied and modified the above for this line. Dont understand it. At all.
        *self.servmsg.lock().unwrap() = Some(ph.hook_server_attrs(
            "PRIVMSG",  // Catch all events.
            ht_core::cb_server,  // Send to Server Callback.
            hexchat_plugin::PRI_NORM,  // Normal Priority.
        ));

        true
    }
}

hexchat_plugin!(HexTwitch);
