//! Interface for Hexchat plugin preferences.
//!
//! Preferences are persistent across Hexchat restarts, and are plugin-specific.
//!
//! The types and constants in this module provide a measure of strong typing
//!     for Hexchat preferences, which are not enforced by Hexchat itself. There
//!     can be no automatic safeguard, however, against simply calling the
//!     relevant Hexchat functions directly.

mod pref_trait;
mod pref_types;
pub mod reward;

pub use pref_trait::*;
pub use pref_types::*;
pub use reward::Reward;


/// Prefix indicating an actual preference setting (as opposed to stored
///     "Reward" data).
#[allow(dead_code)]
const PREFIX: &'static str = "PREF_";


/// Preference: Debug mode for the plugin.
pub const PREF_DEBUG: PrefBool = PrefBool::new("PREF_htdebug");


/// Preference: Whether incoming whispers should be displayed in the current
///     channel in addition to their respective tabs.
pub const PREF_WHISPERS: PrefBool = PrefBool::new("PREF_whispers_in_current");
