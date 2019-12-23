/*
 * Core package for the HexTwitch Rust Plugin.
 *
 * Purely experimental.
 */

#[macro_use]
extern crate hexchat_plugin;

mod ht_core;

use hexchat_plugin::{
    CommandHookHandle, EventAttrs, Plugin, PluginHandle, PrintHookHandle, ServerHookHandle
};
use std::sync::Mutex;

const NAME: &str = env!("CARGO_PKG_NAME");
const DESC: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
struct HexTwitch {
    cmd: Mutex<Option<CommandHookHandle>>,
    print: Mutex<Option<PrintHookHandle>>,
    servmsg: Mutex<Option<ServerHookHandle>>,
    servmsg_2: Mutex<Option<ServerHookHandle>>,
}

impl Plugin for HexTwitch {
    fn init(&self, ph: &mut PluginHandle, _arg: Option<&str>) -> bool {
        ph.register(NAME, DESC, VERSION);

        // Set Command ASDF to print "qwert" to sanity-check that we are loaded.
        *self.cmd.lock().unwrap() = Some(ph.hook_command(
            "asdf",

            |ph, _arg, _arg_eol| {
                ph.print("qwert");
                hexchat_plugin::EAT_ALL
            },

            hexchat_plugin::PRI_NORM,
            Some("prints 'qwert'"),
        ));

        // Hook Print Events into Handler.
        *self.print.lock().unwrap() = Some(ph.hook_print(
            "Channel Message",  // Catch Message Events.
            ht_core::cb_print,  // Send to Print Callback.
            hexchat_plugin::PRI_NORM,  // Normal Priority.
        ));

        // Hook PRIVMSG Server Messages into Store Method of Current Message.
        *self.servmsg.lock().unwrap() = Some(ph.hook_server_attrs(
            "PRIVMSG",  // Catch PRIVMSG events; Chat Messages.

            |ph: &mut PluginHandle, _word, _word_eol, attr: EventAttrs| {
                unsafe { ht_core::CURRENT.put(ph, attr.tags) };

                hexchat_plugin::EAT_NONE
            },

            hexchat_plugin::PRI_NORM,  // Normal Priority.
        ));

        // Hook RAW LINE Server Messages into the general Handler Callback.
        *self.servmsg_2.lock().unwrap() = Some(ph.hook_server_attrs(
            "RAW LINE",  // Catch all events.
            ht_core::cb_server,  // Send to Server Callback.
            hexchat_plugin::PRI_NORM,  // Normal Priority.
        ));

        true
    }
}

hexchat_plugin!(HexTwitch);
