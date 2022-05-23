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
mod icons;
pub mod irc;
mod plugin;
mod prefs;

use plugin::HexTwitch;


const NETWORK: &str = "Twitch";


hexchat::plugin!(HexTwitch);
