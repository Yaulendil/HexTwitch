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

pub use pref_trait::*;
pub use pref_types::*;


/// Plugin-global DRY declarations for preferences stored in Hexchat.
pub struct Pref;

impl Pref {
    /// Prefix indicating an actual preference setting (as opposed to stored
    ///     "Reward" data).
    pub const PREFIX: &'static str = "PREF";

    /// Preference: Debug mode for the plugin.
    pub const DEBUG: PrefBool = PrefBool::new("PREF_htdebug");

    /// Preference: Whether incoming whispers should be displayed in the current
    ///     channel in addition to their respective tabs.
    pub const WHISPERS: PrefBool = PrefBool::new("PREF_whispers_in_current");
}
