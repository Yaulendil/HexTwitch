//! Core package for the HexTwitch Rust Plugin.

#![cfg_attr(feature = "nightly", feature(toowned_clone_into))]

extern crate cached;
#[macro_use]
extern crate hexchat;


/// Execute a command as if typed into Hexchat.
macro_rules! cmd {
    ($text:literal) => { hexchat::send_command($text) };
    ($f:literal, $($t:tt)*) => { hexchat::send_command(&format!($f, $($t)*)) };
}


mod ht_core;
pub mod irc;

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
    send_command,
    WindowEvent,
    WindowEventListener,
};
use ht_core::{
    cb_focus,
    cb_join,
    cb_print,
    cb_server,
    cmd_ht_debug,
    cmd_prediction,
    cmd_reward,
    cmd_title,
    cmd_tjoin,
    cmd_unk_badges,
    cmd_whisper,
    cmd_whisper_here,
};


const NETWORK: &str = "Twitch";


/// Plugin-global DRY declarations for preferences stored in Hexchat.
struct Pref;

impl Pref {
    /// Prefix indicating an actual preference setting (as opposed to stored
    ///     "Reward" data).
    const PREFIX: &'static str = "PREF";

    /// Preference: Debug mode for the plugin.
    const DEBUG: &'static str = "PREF_htdebug";

    /// Preference: Whether incoming whispers should be displayed in the current
    ///     channel in addition to their respective tabs.
    const WHISPERS: &'static str = "PREF_whispers_in_current";
}


enum Hook {
    Command(Command),
    Print(PrintEventListener),
    Server(RawServerEventListener),
    Window(WindowEventListener),
}

impl Hook {
    fn unhook(self) {
        match self {
            Self::Command(handle) => { deregister_command(handle) }
            Self::Print(handle) => { remove_print_event_listener(handle) }
            Self::Server(handle) => { remove_raw_server_event_listener(handle) }
            Self::Window(handle) => { remove_window_event_listener(handle) }
        }
    }
}


macro_rules! hook_print {
    ($hvec:expr, $event:expr, $func:expr) => {
        $hvec.push(Hook::Print(add_print_event_listener(
            $event, Priority::HIGH,
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
        let mut hooks: Vec<Hook> = Vec::with_capacity(17);

        //  Register Plugin Commands, with helptext.
        hooks.push(Hook::Command(register_command(
            "HTDEBUG",
            "Toggle whether unknown UserNotices should show the full plain IRC.",
            Priority::NORMAL,
            cmd_ht_debug,
        )));
        hooks.push(Hook::Command(register_command(
            "PREDICTION",
            "View the current Prediction of the current Twitch Channel.",
            Priority::NORMAL,
            cmd_prediction,
        )));
        hooks.push(Hook::Command(register_command(
            "REWARD",
            "Set the Name of a Custom Reward.\n\n\
                Usage: REWARD <UUID> [<NAME>]",
            Priority::NORMAL,
            cmd_reward,
        )));
        hooks.push(Hook::Command(register_command(
            "TITLE",
            "Set the Title of a Twitch Channel.",
            Priority::NORMAL,
            cmd_title,
        )));
        hooks.push(Hook::Command(register_command(
            "TWITCHJOIN",
            "Join a Channel, but only on the Twitch Network.",
            Priority::NORMAL,
            cmd_tjoin,
        )));
        hooks.push(Hook::Command(register_command(
            "W",
            "Open a Whisper with a Twitch User.\n\n\
                Usage: W <username> [<message>]",
            Priority::NORMAL,
            cmd_whisper,
        )));
        hooks.push(Hook::Command(register_command(
            "WHISPERHERE",
            "Toggle whether Twitch Whispers should be duplicated in the current Tab.",
            Priority::NORMAL,
            cmd_whisper_here,
        )));
        hooks.push(Hook::Command(register_command(
            "UNKNOWNS",
            "Display unknown Badge Keys that have been seen.",
            Priority::NORMAL,
            cmd_unk_badges,
        )));

        for cmd in &[
            "Twitch",
            "\"Twitch/Toggle USERNOTICE Debug\" HTDEBUG",
            "\"Twitch/Toggle in-channel Whispers\" WHISPERHERE",
        ] { send_command(&format!("MENU ADD {}", cmd)); }

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
        hooks.push(Hook::Server(add_raw_server_event_listener(
            "RAW LINE",
            Priority::NORMAL,
            cb_server,
        )));

        //  Hook Tab Focus events.
        hooks.push(Hook::Window(add_window_event_listener(
            WindowEvent::FOCUS_TAB,
            Priority::NORMAL,
            cb_focus,
        )));

        //  Report loadedness.
        print_plain(&format!("{} {} loaded.", Self::NAME, Self::VERSION));

        Self { hooks }
    }
}

impl Drop for HexTwitch {
    fn drop(&mut self) { self.hooks.drain(..).for_each(Hook::unhook); }
}


plugin!(HexTwitch);
