//! Interface for Hexchat plugin preferences.
//!
//! Preferences are persistent across Hexchat restarts, and are plugin-specific.
//!
//! The types and constants in this module provide a measure of strong typing
//!     for Hexchat preferences, which are not enforced by Hexchat itself. There
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


/// Preference: Debug mode for the plugin.
pub const PREF_DEBUG: PrefMigrating<PrefBool> = PrefMigrating {
    new: PrefBool::new(pref!("debug")),
    old: PrefBool::new(pref!("htdebug")),
};


pub const PREF_FOLLOW_HOSTS: PrefBool = PrefBool::new(pref!("follow_hosts"));


/// Preference: Whether incoming whispers should be displayed in the current
///     channel in addition to their respective tabs.
pub const PREF_WHISPERS: PrefBool = PrefBool::new(pref!("whispers_in_current"));


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
