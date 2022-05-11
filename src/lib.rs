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
mod plugin;

use plugin::HexTwitch;


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


hexchat::plugin!(HexTwitch);
