/*
 * Core package for the HexTwitch Rust Plugin.
 */

#[macro_use]
extern crate hexchat;

mod ht_core;


use hexchat::{
    add_print_event_listener,
    add_raw_server_event_listener,
    Command,
    deregister_command,
    EatMode,
    Plugin,
    plugin,
    print_plain,
    PrintEvent,
    PrintEventListener,
    Priority,
    RawServerEventListener,
    register_command,
    remove_print_event_listener,
    remove_raw_server_event_listener,
};

use ht_core::{cb_print, cb_server};


enum Hook {
    CommandHook(Command),
    PrintHook(PrintEventListener),
    ServerHook(RawServerEventListener),
}


macro_rules! hook_print {
    ($hvec:expr, $event:expr, $func:expr) => {
        $hvec.push(Hook::PrintHook(add_print_event_listener(
            $event,  // Catch Message Events.
            Priority::HIGH,
            |word, _dt| $func($event, word),
        )));
    }
}


#[derive(Default)]
struct HexTwitch { hooks: Vec<Hook> }

impl Plugin for HexTwitch {
    const NAME: &'static str = env!("CARGO_PKG_NAME");
    const DESC: &'static str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn new() -> Self {
        let mut hooks: Vec<Hook> = Vec::new();

        // Set Command ASDF to print "qwert" to sanity-check that we are loaded.
        hooks.push(Hook::CommandHook(register_command(
            "asdf",
            "prints 'qwert'",
            Priority::NORMAL,
            |_arg| {
                print_plain("qwert");
                EatMode::All
            },
        )));

        // Hook Print Events into Handler.
        hook_print!(hooks, PrintEvent::CHANNEL_MESSAGE, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_ACTION, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_MSG_HILIGHT, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_ACTION_HILIGHT, cb_print);
        hook_print!(hooks, PrintEvent::YOUR_MESSAGE, cb_print);
        hook_print!(hooks, PrintEvent::YOUR_ACTION, cb_print);

        // Hook RAW LINE Server Messages into the general Handler Callback.
        hooks.push(Hook::ServerHook(add_raw_server_event_listener(
            "RAW LINE",  // Catch all events.
            Priority::NORMAL,
            cb_server,  // Send to Server Callback.
        )));

        let new: Self = Self { hooks };

        print_plain(&format!("{} {} loaded", Self::NAME, Self::VERSION));
        new
    }
}


impl Drop for HexTwitch {
    fn drop(&mut self) {
        for hopt in self.hooks.drain(..) {
            match hopt {
                Hook::CommandHook(handle) => { deregister_command(handle) }
                Hook::PrintHook(handle) => { remove_print_event_listener(handle) }
                Hook::ServerHook(handle) => { remove_raw_server_event_listener(handle) }
            }
        }
    }
}


plugin!(HexTwitch);
