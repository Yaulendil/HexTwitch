use std::str::FromStr;
use super::pref_trait::*;


#[cfg(feature = "strict-reward-uuids")]
//  TODO: This is overkill. Should just use a UUID library instead.
fn is_uuid(name: &str) -> bool {
    use std::str::Chars;

    let chars: &mut Chars = &mut name.chars();

    fn expect_delim(iter: &mut Chars) -> bool {
        iter.next() == Some('-')
    }

    fn expect_digits(iter: &mut Chars, number: usize) -> bool {
        for _ in 0..number {
            if !matches!(iter.next(), Some('0'..='9' | 'a'..='f' | 'A'..='F')) {
                return false;
            }
        }
        true
    }

    expect_digits(chars, 8)
        && expect_delim(chars) && expect_digits(chars, 4)
        && expect_delim(chars) && expect_digits(chars, 4)
        && expect_delim(chars) && expect_digits(chars, 4)
        && expect_delim(chars) && expect_digits(chars, 12)
        && chars.next().is_none()
}

#[cfg(not(feature = "strict-reward-uuids"))]
fn is_uuid(name: &str) -> bool {
    !name.is_empty() && !name.starts_with(super::PREFIX)
}


pub struct Reward(String);

impl Reward {
    pub fn new(mut uuid: String) -> Option<Self> {
        if is_uuid(&uuid) {
            uuid.make_ascii_lowercase();
            Some(Self(uuid))
        } else {
            None
        }
    }

    pub fn get_all() -> impl Iterator<Item=Self> {
        hexchat::get_prefs().into_iter().filter_map(Self::new)
    }

    pub fn id(&self) -> &str { &self.name() }
}

impl FromStr for Reward {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_uuid(s) {
            Ok(Self(s.to_lowercase()))
        } else {
            Err(())
        }
    }
}

impl HexPref for Reward {
    fn name(&self) -> &str { &self.0 }
}

impl HexPrefGet for Reward {
    type Output = String;

    fn get(&self) -> Option<Self::Output> {
        hexchat::get_pref_string(self.name())
    }
}

impl<T: AsRef<str>> HexPrefSet<T> for Reward {
    fn set(&self, value: T) -> Result<(), ()> {
        hexchat::set_pref_string(self.name(), value.as_ref())
    }
}

impl HexPrefUnset for Reward {}


#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "strict-reward-uuids")]
    const SAMPLES: &[(u32, &str, bool)] = {
        &[
            (line!(), "", false),
            (line!(), "PREF_", false),
            (line!(), "PREF_asdfqwert", false),
            (line!(), "asdf qwert", false),

            //  Test for incorrect segment lengths.
            (line!(), "0000000-000-000-000-00000000000", false),
            (line!(), "0000000-0000-0000-0000-000000000000", false),
            (line!(), "00000000-000-0000-0000-000000000000", false),
            (line!(), "00000000-0000-000-0000-000000000000", false),
            (line!(), "00000000-0000-0000-000-000000000000", false),
            (line!(), "00000000-0000-0000-0000-00000000000", false),
            (line!(), "00000000-0000-0000-0000-000000000000", true),
            (line!(), "00000000-0000-0000-0000-0000000000000", false),
            (line!(), "00000000-0000-0000-00000-000000000000", false),
            (line!(), "00000000-0000-00000-0000-000000000000", false),
            (line!(), "00000000-00000-0000-0000-000000000000", false),
            (line!(), "000000000-0000-0000-0000-000000000000", false),

            //  Test all valid characters.
            (line!(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", true),
            (line!(), "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA", true),
            (line!(), "01234567-89ab-cdef-0123-456789abcdef", true),
            (line!(), "01234567-89AB-CDEF-0123-456789ABCDEF", true),
            (line!(), "01234567-89AB-cdEF-0123-456789aBCdEf", true),

            //  Test for invalid characters.
            (line!(), "01234567-89ab-cdef-0123-456789abcxyz", false),
            (line!(), "01234567-89AB-CDEF-0123-456789ABCXYZ", false),

            //  Test for invalid delimiters.
            (line!(), "01234567 89ab cdef 0123 456789abcdef", false),
            (line!(), "0123-456789ab-cdef-0123-456789abcdef", false),
            (line!(), "-012345678-9ab-cdef-0123-456789abcdef", false),
            (line!(), "012345678-9ab-cdef-0123-456789abcdef-", false),
            (line!(), "01234567-89ab-cdef-01234567-89abcdef", false),
            (line!(), "0123456789abcdef0123456789abcdef", false),
        ]
    };
    #[cfg(not(feature = "strict-reward-uuids"))]
    const SAMPLES: &[(u32, &str, bool)] = {
        &[
            (line!(), "", false),
            (line!(), "PREF_", false),
            (line!(), "PREF_asdfqwert", false),

            //  Test for correct segment length.
            (line!(), "00000000-0000-0000-0000-000000000000", true),

            //  Test all valid characters.
            (line!(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", true),
            (line!(), "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA", true),
            (line!(), "01234567-89ab-cdef-0123-456789abcdef", true),
            (line!(), "01234567-89AB-CDEF-0123-456789ABCDEF", true),
            (line!(), "01234567-89AB-cdEF-0123-456789aBCdEf", true),
        ]
    };

    #[test]
    fn test_uuids() {
        for &(line, uuid, valid) in SAMPLES {
            assert_eq!(
                is_uuid(uuid), valid,
                "String at line {line} ({uuid:?}) is incorrectly determined \
                {to_be} a valid UUID.",
                to_be = if valid {
                    "NOT to be"
                } else {
                    "to be"
                }
            );
        }
    }
}
