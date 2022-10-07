//! Interface for Hexchat plugin preferences.
//!
//! Preferences are persistent across Hexchat restarts, and are plugin-specific.
//!
//! The types and constants in this module provide a measure of strong typing
//!     for Hexchat preferences, which is not enforced by Hexchat itself. There
//!     can be no automatic safeguard, however, against simply calling the
//!     relevant Hexchat functions directly.

mod migrations;
mod pref_trait;
mod pref_types;
pub mod reward;

use migrations::*;
pub use pref_trait::*;
pub use pref_types::*;
pub use reward::Reward;


macro_rules! pref {($($t:tt)*) => {concat!("PREF_", $($t)*)}}


/// Prefix indicating an actual preference setting (as opposed to stored
///     "Reward" data).
#[allow(dead_code)]
const PREFIX: &'static str = pref!();


// pub const PREF_API_CLIENT: PrefStr = PrefStr::new(pref!("api_client"));
pub const PREF_API_OAUTH2: PrefStr = PrefStr::new(pref!("api_token"));


/// Preference: Whether Twitch "Announcement" messages should be distinguished
///     with colors.
pub const PREF_ANNOUNCE: PrefBool = PrefBool::new(pref!("color_announcements"));


/// Preference: Debug mode for the plugin.
pub const PREF_DEBUG: PrefMigrating<PrefBool> = PrefMigrating {
    new: PrefBool::new(pref!("debug")),
    old: PrefBool::new(pref!("htdebug")),
};


pub const PREF_FOLLOW_HOSTS: PrefBool = PrefBool::new(pref!("follow_hosts"));


/// Preference: Whether incoming whispers should be displayed in the current
///     channel in addition to their respective tabs.
pub const PREF_WHISPERS: PrefBool = PrefBool::new(pref!("whispers_in_current"));


/// Set preferences to initial values, printing warnings if they cannot be set.
pub fn init_prefs() {
    fn init_report<T>(pref: impl HexPrefGet + HexPrefSet<T>, value: T) {
        if let Err(()) = pref.init(value) {
            hexchat::print_plain(&format!(
                "Failed to set initial value for preference: {}",
                pref.name(),
            ));
        }
    }

    init_report(PREF_ANNOUNCE, true);
    init_report(PREF_DEBUG, false);
    init_report(PREF_FOLLOW_HOSTS, false);
    init_report(PREF_WHISPERS, false);
}


/// Perform all necessary Preference migrations, printing a report for each one.
#[allow(dead_code)]
pub fn migrate_prefs() {
    fn migrate_report<New, Old>(pref: PrefMigrating<New, Old>) where
        New: HexPrefGet + HexPrefSet<<Old as HexPrefGet>::Output>,
        Old: HexPrefGet + HexPrefUnset,
    {
        let report = match pref.migrate() {
            Ok(MigrateAction::NoOldValue) => None,
            Ok(MigrateAction::OldValueCleared) => Some(format!(
                "Cleared outdated HexTwitch preference {old:?}.",
                old = pref.old.name(),
            )),
            Ok(MigrateAction::OldValueMoved) => Some(format!(
                "HexTwitch preference {old:?} has been renamed to {new:?}.",
                new = pref.new.name(),
                old = pref.old.name(),
            )),
            Err(MigrateFail::CannotUnsetOld) => Some(format!(
                "Failed to clear outdated HexTwitch preference {old:?}.",
                old = pref.old.name(),
            )),
            Err(MigrateFail::CannotSetNew) => Some(format!(
                "Failed to rename HexTwitch preference {old:?} to {new:?}.",
                new = pref.new.name(),
                old = pref.old.name(),
            )),
        };

        if let Some(text) = report {
            hexchat::print_plain(&text);
        }
    }

    migrate_report(PREF_DEBUG);
}
