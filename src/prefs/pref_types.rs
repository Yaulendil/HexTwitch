use super::pref_trait::*;


/// A preference with a simple boolean value.
///
/// Hexchat does not actually support boolean preference values, so this is a
///     thin wrapper for an integer preference. Zero is interpreted as `false`,
///     and all other values are interpreted as `true`.
pub struct PrefBool(&'static str);

impl PrefBool {
    #[allow(dead_code)]
    pub(super) const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

impl HexPref for PrefBool {
    fn name(&self) -> &str { self.0 }
}

impl HexPrefGet for PrefBool {
    type Output = bool;

    fn get(&self) -> Option<Self::Output> {
        let n: u32 = hexchat::get_pref_int(self.name())?;

        Some(n != 0)
    }
}

impl HexPrefSet<bool> for PrefBool {
    fn set(&self, value: bool) -> Result<(), ()> {
        hexchat::set_pref_int(self.name(), value as _)
    }
}

impl HexPrefUnset for PrefBool {}


/// A preference with a 32-bit unsigned integer value.
pub struct PrefInt(&'static str);

impl PrefInt {
    #[allow(dead_code)]
    pub(super) const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

impl HexPref for PrefInt {
    fn name(&self) -> &str { self.0 }
}

impl HexPrefGet for PrefInt {
    type Output = u32;

    fn get(&self) -> Option<Self::Output> {
        hexchat::get_pref_int(self.name())
    }
}

impl HexPrefSet<u32> for PrefInt {
    fn set(&self, value: u32) -> Result<(), ()> {
        hexchat::set_pref_int(self.name(), value)
    }
}

impl HexPrefUnset for PrefInt {}


/// A preference with a string value.
pub struct PrefStr(&'static str);

impl PrefStr {
    #[allow(dead_code)]
    pub(super) const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

impl HexPref for PrefStr {
    fn name(&self) -> &str { self.0 }
}

impl HexPrefGet for PrefStr {
    type Output = String;

    fn get(&self) -> Option<Self::Output> {
        hexchat::get_pref_string(self.name())
    }
}

impl<T: AsRef<str>> HexPrefSet<T> for PrefStr {
    fn set(&self, value: T) -> Result<(), ()> {
        hexchat::set_pref_string(self.name(), value.as_ref())
    }
}

impl HexPrefUnset for PrefStr {}
