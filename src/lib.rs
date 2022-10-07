//! Core package for the HexTwitch Rust Plugin.

extern crate cached;
#[macro_use]
extern crate hexchat;

#[macro_use]
mod macros;

mod api;
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
