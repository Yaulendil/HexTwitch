/*
 * Core package for the HexTwitch Rust Plugin.
 */

#![feature(option_result_contains, test, toowned_clone_into)]

#[macro_use]
extern crate hexchat;

mod ht_core;


use hexchat::{
    add_print_event_listener,
    add_raw_server_event_listener,
    add_window_event_listener,
    Command,
    deregister_command,
    Plugin,
    print_plain,
    PrintEvent,
    PrintEventListener,
    Priority,
    RawServerEventListener,
    register_command,
    remove_print_event_listener,
    remove_raw_server_event_listener,
    remove_window_event_listener,
    WindowEvent,
    WindowEventListener,
};

use ht_core::{
    cb_focus,
    cb_join,
    cb_print,
    cb_server,
    cmd_reward,
    cmd_title,
    cmd_tjoin,
    cmd_whisper,
    cmd_whisper_here,
};


enum Hook {
    CommandHook(Command),
    PrintHook(PrintEventListener),
    ServerHook(RawServerEventListener),
    WindowHook(WindowEventListener),
}


macro_rules! hook_print {
    ($hvec:expr, $event:expr, $func:expr) => {
        $hvec.push(Hook::PrintHook(add_print_event_listener(
            $event,
            Priority::HIGH,
            |word, _dt| $func($event, word),
        )));
    }
}


#[derive(Default)]
struct HexTwitch(Vec<Hook>);

impl Plugin for HexTwitch {
    const NAME: &'static str = env!("CARGO_PKG_NAME");
    const DESC: &'static str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn new() -> Self {
        let mut hooks: Vec<Hook> = Vec::with_capacity(14);

        //  Register Plugin Commands, with helptext.
        hooks.push(Hook::CommandHook(register_command(
            "REWARD",
            "Set the Name of a Custom Reward.\n\n\
                Usage: REWARD <UUID> [<NAME>]",
            Priority::NORMAL,
            cmd_reward,
        )));
        hooks.push(Hook::CommandHook(register_command(
            "TITLE",
            "Set the Title of a Twitch Channel.",
            Priority::NORMAL,
            cmd_title,
        )));
        hooks.push(Hook::CommandHook(register_command(
            "TWITCHJOIN",
            "Join a Channel, but only on the Twitch Network.",
            Priority::NORMAL,
            cmd_tjoin,
        )));
        hooks.push(Hook::CommandHook(register_command(
            "W",
            "Open a Whisper with a Twitch User.\n\n\
                Usage: W <username> [<message>]",
            Priority::NORMAL,
            cmd_whisper,
        )));
        hooks.push(Hook::CommandHook(register_command(
            "WHISPERHERE",
            "Toggle whether Twitch Whispers should be duplicated in the current Tab.",
            Priority::NORMAL,
            cmd_whisper_here,
        )));

        //  Hook for User Joins.
        hook_print!(hooks, PrintEvent::JOIN, cb_join);

        //  Hooks for User Messages.
        hook_print!(hooks, PrintEvent::CHANNEL_MESSAGE, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_ACTION, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_MSG_HILIGHT, cb_print);
        hook_print!(hooks, PrintEvent::CHANNEL_ACTION_HILIGHT, cb_print);
        hook_print!(hooks, PrintEvent::YOUR_MESSAGE, cb_print);
        hook_print!(hooks, PrintEvent::YOUR_ACTION, cb_print);

        //  Hook RAW LINE Server Messages into the general Handler Callback.
        hooks.push(Hook::ServerHook(add_raw_server_event_listener(
            "RAW LINE",
            Priority::NORMAL,
            cb_server,
        )));

        //  Hook Tab Focus events.
        hooks.push(Hook::WindowHook(add_window_event_listener(
            WindowEvent::FOCUS_TAB,
            Priority::NORMAL,
            cb_focus,
        )));

        //  Report loadedness.
        print_plain(&format!("{} {} loaded", Self::NAME, Self::VERSION));

        Self(hooks)
    }
}

impl Drop for HexTwitch {
    fn drop(&mut self) {
        for hopt in self.0.drain(..) {
            match hopt {
                Hook::CommandHook(handle) => { deregister_command(handle) }
                Hook::PrintHook(handle) => { remove_print_event_listener(handle) }
                Hook::ServerHook(handle) => { remove_raw_server_event_listener(handle) }
                Hook::WindowHook(handle) => { remove_window_event_listener(handle) }
            }
        }
    }
}


plugin!(HexTwitch);
