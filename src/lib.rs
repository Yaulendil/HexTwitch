/*
 * Core package for the HexTwitch Rust Plugin.
 *
 * Purely experimental.
 */

#[macro_use]
extern crate hexchat;

mod ht_core;


use std::mem::replace;
use std::sync::Mutex;

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


//enum Hook {
//    CommandHook(Command),
//    PrintHook(PrintEventListener),
//    ServerHook(RawServerEventListener),
//}


#[derive(Default)]
struct HexTwitch { /*hooks: Mutex<Vec<Hook>>*/ }

impl Plugin for HexTwitch {
    const NAME: &'static str = env!("CARGO_PKG_NAME");
    const DESC: &'static str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn new() -> Self {
        print_plain(&format!("Loading {} {}", Self::NAME, Self::VERSION));

        // Set Command ASDF to print "qwert" to sanity-check that we are loaded.
//        let cmd = Hook::CommandHook(
        register_command(
            "asdf",
            "prints 'qwert'",
            Priority::NORMAL,
            |_arg| {
                print_plain("qwert");
                EatMode::All
            },
        );

        // Hook Print Events into Handler.
//        let print = Hook::PrintHook(
        add_print_event_listener(
            PrintEvent::CHANNEL_MESSAGE,  // Catch Message Events.
            Priority::NORMAL,
            ht_core::cb_print,  // Send to Print Callback.
        );

        // Hook RAW LINE Server Messages into the general Handler Callback.
//        let servmsg = Hook::ServerHook(
        add_raw_server_event_listener(
            "RAW LINE",  // Catch all events.
            Priority::NORMAL,
            ht_core::cb_server,  // Send to Server Callback.
        );

        let new: Self = Self { /*hooks: Mutex::new(vec![cmd, print, servmsg])*/ };

        print_plain(&format!("{} loaded.", Self::NAME));
        new
    }
}


//impl Drop for HexTwitch {
//    fn drop(&mut self) {
//        let iter: Vec<Hook> = replace(self.hooks.get_mut().unwrap(), vec![]);
//        for hopt in iter {
//            match hopt {
//                Hook::CommandHook(handle) => { deregister_command(handle) }
//                Hook::PrintHook(handle) => { remove_print_event_listener(handle) }
//                Hook::ServerHook(handle) => { remove_raw_server_event_listener(handle) }
//            }
//        }
//    }
//}


plugin!(HexTwitch);
