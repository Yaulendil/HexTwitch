use cached::proc_macro::cached;
use hexchat::{print_event, PrintEvent};
use parking_lot::RwLock;
use std::{
    collections::{
        hash_map::{Entry, HashMap},
        HashSet,
    },
    ops::Deref,
};
use super::{
    super::irc::split_at_char,
    tabs::TABCOLORS,
};


/// Channel Events: Subscriptions, Highlighted Messages, etc.
pub const EVENT_ALERT: PrintEvent = PrintEvent::WHOIS_SERVER_LINE;
/// Links to other Channels, like Hosting.
pub const EVENT_CHANNEL: PrintEvent = PrintEvent::CHANNEL_URL;
/// Red "error" text: Things going wrong, or people being banned.
pub const EVENT_ERR: PrintEvent = PrintEvent::SERVER_ERROR;
/// Typical events.
pub const EVENT_NORMAL: PrintEvent = PrintEvent::MOTD;
/// Reward Events: Bits and custom Points Rewards.
pub const EVENT_REWARD: PrintEvent = PrintEvent::WHOIS_AUTHENTICATED;


/// Echo: Print an event to HexChat in the current Channel, and color the tab.
///
/// Input: `PrintEvent`, `&[impl AsRef<str>]`, `u8`
#[inline]
pub fn echo(event: PrintEvent, args: &[impl AsRef<str>], tab_color: u8) {
    print_event(event, args);
    TABCOLORS.lock().color(tab_color);
}


pub fn alert_basic(message: &str) {
    echo(EVENT_NORMAL, &[message], 1);
}


pub fn alert_error(message: &str) {
    echo(EVENT_ERR, &[message], 1);
}


pub fn alert_subscription(message: &str) {
    echo(EVENT_ALERT, &["SUBSCRIPTION", message], 2);
}


pub fn alert_sub_upgrade(message: &str) {
    echo(EVENT_ALERT, &["UPGRADE", message], 2);
}


/// BITS: Badge characters for Bits. If a User has a Bits Badge, the User is
///     given the `char` corresponding to the last value found here which is
///     LESS THAN OR EQUAL TO the Rank of the Badge.
/// NOTE: if any value here is not greater than the previous one, it and
///     subsequent pairs will not be considered in the correct order.
static BITS: &[(usize, char)] = &[
    (0, '▴'),
    (100, '⬧'),
    (1_000, '⬠'),
    (5_000, '⬡'),
    (10_000, '🟋'),
    // (25_000, '?'),
    // (50_000, '?'),
    // (75_000, '?'),
    (100_000, '🟎'),
    // (200_000, '?'),
    // (300_000, '?'),
    // (400_000, '?'),
    // (500_000, '?'),
    // (600_000, '?'),
    // (700_000, '?'),
    // (800_000, '?'),
    // (900_000, '?'),
    // (1_000_000, '?'),
];
/// SUBS: Badge characters for Subscribers. If a User has a Sub Badge, the User
///     is given the `char` corresponding to the last value found here which is
///     LESS THAN OR EQUAL TO the Rank of the Badge.
/// NOTE: if any value here is not greater than the previous one, it and
///     subsequent pairs will not be considered in the correct order.
static SUBS: &[(usize, char)] = &[
    (0, '⓵'),
    (3, '⓷'),
    (6, '⓺'),
    (9, '⓽'),
    (12, 'ⅰ'),
    (24, 'ⅱ'),
    (36, 'ⅲ'),
    (48, 'ⅳ'),
    (60, 'ⅴ'),
    (72, 'ⅵ'),
    (84, 'ⅶ'),
    (96, 'ⅷ'),
    (108, 'ⅸ'),
    (120, 'ⅹ'),
    (132, 'ⅺ'),
    (144, 'ⅻ'),
    (156, '⓭'),
    (168, '⓮'),
    (180, '⓯'),
    (192, '⓰'),
    (204, '⓱'),
    (216, '⓲'),
    (228, '⓳'),
    (240, '⓴'),
    (252, '⁑'),
];
// /// GIFTS: Badge characters for Subscription Gifters. If a User has a Gifter
// /// Badge, the User is given the `char` corresponding to the last value found
// /// here which is LESS THAN OR EQUAL TO the Rank of the Badge.
// /// NOT E: if any value here is not greater than the previous one, it and
// ///     subsequent pairs will not be considered in the correct order.
// static GIFTS: &[(usize, char)] = &[
//     (0, ':'),
//     // (5, '?'),
//     // (10, '?'),
//     // (25, '?'),
//     // (50, '?'),
//     // (100, '?'),
//     // (250, '?'),
//     // (500, '?'),
//     // (1_000, '?'),
// ];


safe_static! {
    pub static lazy BADGES_UNK: RwLock<HashSet<String>> = Default::default();
}


/// Determine if a Badge title is probably a game-specific badge.
///
/// There are dozens of these, so it is not worth assigning each one a unique
///     character.
fn is_game_badge(name: &str) -> bool {
    //  Find the last underscore.
    match name.rfind('_') {
        Some(idx) => {
            //  This is probably a Game Badge if it does not end with an
            //      underscore, and all characters after the underscore are
            //      numeric.
            name[idx + 1..].parse::<usize>().is_ok()
        }
        None => false,
    }
}


fn get_badge(class: &str, rank: &str) -> char {
    match class {
        "broadcaster"       /**/ => '🜲',
        "staff"             /**/ => '🜨',
        "admin"             /**/ => '⛨',

        "moderator"         /**/ => '🗡',
        "subscriber"        /**/ => highest(rank.parse().unwrap_or(0), &SUBS),
        "vip"               /**/ => '⚑',
        "founder"           /**/ => 'ⲷ',

        "sub-gift-leader"   /**/ => '⁘',
        // "sub-gifter"        /**/ => highest(rank.parse().unwrap_or(0), &GIFTS),
        "sub-gifter"        /**/ => ':',
        "bits-charity"      /**/ => '🝔',
        "bits-leader"       /**/ => '❖',
        "bits"              /**/ => highest(rank.parse().unwrap_or(0), &BITS),
        "hype-train"        /**/ => '.',

        "partner"           /**/ => '✓',
        "turbo"             /**/ => '+',
        "premium"           /**/ => '±',

        "glhf-pledge"       /**/ => '~',
        "anonymous-cheerer" /**/ => '*',

        "ambassador"        /**/ => 'a',
        "glitchcon2020"     /**/ => 'g',
        "predictions"       /**/ => prediction_badge(rank),

        s if s.starts_with("twitchcon") => 'c',
        s if s.starts_with("overwatch-league-insider") => 'w',
        s if is_game_badge(s) => 'G',
        s => {
            BADGES_UNK.write().get_or_insert_owned(s);

            '?'
        }
    }
}


fn highest(max: usize, seq: &[(usize, char)]) -> char {
    match seq.binary_search_by(|pair: &(usize, char)| pair.0.cmp(&max)) {
        Err(0) => '¿',
        Err(idx) => unsafe { seq.get_unchecked(idx - 1).1 }
        Ok(idx) => unsafe { seq.get_unchecked(idx).1 }
    }
}


fn prediction_badge(pred: &str) -> char {
    match pred {
        //  These two will suffice if Twitch never adds more possibilities. This
        //      does not seem likely, but it is possible.
        "blue-1" => '⧮',
        "pink-2" => '⧯',
        _ => {
            BADGES_UNK.write().get_or_insert(format!("predictions/{}", pred));

            'p'
        }
    }
}


/// Badges: A Struct storing the Input and Output of the process of breaking
///     down a badge value. This effectively serves the purpose of a Cached
///     Function.
#[derive(Clone, Default)]
pub struct Badges {
    input: (String, String),
    pub output: String,
}

impl Badges {
    /// A placeholder Badge string for the User when a UserState has not been
    ///     received.
    pub const NONE: &'static str = "_ ";

    /// Break down a string to find the final set of characters. The original
    ///     will be stored.
    ///
    /// Input: `String`, `String`
    /// Return: `Badges`
    fn from_str(badges: String, meta: String) -> Self {
        let mut output: String = String::with_capacity(16);

        if !badges.is_empty() {
            for pair in badges.split(',') {
                //  Twitch now provides the number of months attached to a Sub
                //      Badge separately, in the `badge-info` Tag. The number
                //      attached directly to the Badge itself will only reflect
                //      the correct number of months if the channel has a custom
                //      icon set for the tier.
                if pair.starts_with("subscriber/") && !meta.is_empty() {
                    for pair_info in meta.split(',') {
                        if pair_info.starts_with("subscriber/") {
                            output.push(get_badge("subscriber", &pair_info[11..]));
                            break;
                        }
                    }
                } else {
                    let (class, rank) = split_at_char(pair, '/');
                    output.push(get_badge(class, rank));
                }
            }

            if !output.is_empty() { output.push(' '); }
        }

        // output.shrink_to_fit();

        Self { input: (badges, meta), output }
    }
}


impl<T> AsRef<T> for Badges where
    String: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.output.as_ref()
    }
}


impl Deref for Badges {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}


/// Passthrough function required for caching.
#[cached(size = 50)]
pub fn badge_parse(input: String, info: String) -> Badges {
    Badges::from_str(input, info)
}


/// States: Effectively a guarded wrapper for a HashMap. Stores the Badges for
///     the User in each Channel.
#[derive(Default)]
pub struct States { inner: HashMap<String, Badges> }

impl States {
    /// Get the Badges for the User in a given Channel.
    ///
    /// Input: `&str`
    /// Return: `&str`
    pub fn get(&self, channel: &str) -> &str {
        self.inner.get(channel)
            .map(Deref::deref)
            .unwrap_or(Badges::NONE)
    }

    /// Set the Badges for the User in a given Channel. This is mostly just a
    ///     guarded passthrough to the internal HashMap, but with one
    ///     significant difference: If the current value for the given Channel
    ///     in the Map was created from the same input as has been given here,
    ///     the input is NOT evaluated again.
    ///
    /// Returns a Reference to the new `Badges` if there was a change, `None`
    ///     otherwise.
    ///
    /// Input: `String`, `String`, `String`
    /// Output: `Option<&Badges>`
    pub fn set(&mut self, channel: String, bstr: String, meta: String)
               -> Option<&Badges>
    {
        match self.inner.entry(channel) {
            Entry::Vacant(entry) => Some(entry.insert(badge_parse(bstr, meta))),
            Entry::Occupied(entry) => {
                let badges: &mut Badges = entry.into_mut();

                if badges.input.0 != bstr || badges.input.1 != meta {
                    *badges = badge_parse(bstr, meta);
                    Some(badges)
                } else {
                    //  Channel is in Map, with the same Badges.
                    None
                }
            }
        }
    }
}

safe_static! {
    pub static lazy USERSTATE: RwLock<States> = Default::default();
}


// #[cfg(test)]
// mod tests_badge {
//     extern crate test;
//
//     use crate::ht_core::irc::{
//         Message,
//         tests_irc::SAMPLES,
//     };
//     use super::*;
//     use test::Bencher;
//
//     /// Benchmark performance of reading `Badges` from `Message`s.
//     #[bench]
//     fn bench_badges(b: &mut Bencher) {
//         for raw in SAMPLES {
//             let msg: Message = raw.parse().expect("Parse Failed");
//
//             if msg.has_tags() {
//                 b.iter(|| {
//                     let _b: Badges = badge_parse(
//                         msg.get_tag("badges").unwrap_or_default(),
//                         msg.get_tag("badge-info").unwrap_or_default(),
//                     );
//                 });
//             }
//         }
//     }
// }
