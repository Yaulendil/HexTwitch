//! Core package for the HexTwitch Rust Plugin.

#![cfg_attr(feature = "nightly", feature(toowned_clone_into))]

extern crate cached;
#[macro_use]
extern crate hexchat;


/// Execute a command as if typed into HexChat.
macro_rules! cmd {
    // ($($t:tt)*) => { ::hexchat::send_command(&format!($($t)*)) };
    ($text:literal) => { hexchat::send_command($text) };
    ($f:literal, $($t:tt)*) => { hexchat::send_command(&format!($f, $($t)*)) };
}

/// Print text to the Twitch network tab in HexChat.
macro_rules! twitch_print {
    ($($t:tt)*) => {
        ::hexchat::send_command(&format!(
            "DOAT Twitch ECHO {}",
            format_args!($($t)*),
        ))
    };
}


mod ht_core;
mod icons;
pub mod irc;
mod plugin;
mod prefs;

use plugin::HexTwitch;


const NETWORK: &str = "Twitch";
const PLUGIN_INFO: &str = concat!("\
About HexTwitch (", env!("CARGO_PKG_VERSION"), "):
\
HexTwitch is a HexChat plugin implementing Twitch.tv functionality, written in \
Rust. Using data from IRC Tags, it displays channel events such as Bits and \
Subscriptions, and applies Twitch badges to users in the form of Unicode \
characters.
\
Repository: https://github.com/Yaulendil/HexTwitch
");


hexchat::plugin!(HexTwitch);
