use std::collections::HashMap;

use hexchat::{print_event, PrintEvent};
use parking_lot::RwLock;

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
    TABCOLORS.write().color(tab_color);
}


/// BADGE_NONE: A placeholder Badge string for the User when a UserState has not
///     been received.
const BADGE_NONE: &str = "_ ";
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
    (0, '①'),
    (3, '③'),
    (6, '⑥'),
    (9, '⑨'),
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


fn get_badge(class: &str, rank: &str) -> char {
    match class {
        "broadcaster"       /**/ => '🜲',
        "staff"             /**/ => '🜨',
        "admin"             /**/ => '🜶',

        "moderator"         /**/ => '🗡',  // ⛨?
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

        s if s.starts_with("twitchcon") => 'c',
        _ => '?',
    }
}


fn highest(max: usize, seq: &[(usize, char)]) -> char {
    let mut out: char = '¿';

    for pair in seq {
        if pair.0 <= max {
            out = pair.1;
        } else {
            break;
        }
    }

    out
}


/// Badges: A Struct storing the Input and Output of the process of breaking
///     down a badge value. This effectively serves the purpose of a Cached
///     Function.
#[derive(Default)]
pub struct Badges {
    input: String,
    pub output: String,
}

impl Badges {
    /// Break down a string to find the final set of characters. The original
    ///     will be stored.
    ///
    /// Input: `&str`, `&str`
    /// Return: `Badges`
    pub fn from_str(input: &str, info: &str) -> Self {
        let mut output: String = String::with_capacity(16);

        if !input.is_empty() {
            for pair in input.split(',') {
                let (class, rank) = split_at_char(pair, '/');

                //  Twitch now provides the number of months attached to a Sub
                //      Badge separately, in the `badge-info` Tag. The number
                //      attached directly to the Badge itself will only reflect
                //      the correct number of months if the channel has a custom
                //      icon set for the tier.
                if class == "subscriber" && !info.is_empty() {
                    for pair_info in info.split(',') {
                        if pair_info.starts_with("subscriber/") {
                            output.push(get_badge(class, &pair_info[11..]));
                            break;
                        }
                    }
                } else {
                    output.push(get_badge(class, rank));
                }
            }

            if !output.is_empty() { output.push(' '); }
            // output.shrink_to_fit();
        }
        Self { input: input.to_owned(), output }
    }
}


/// States: Effectively a Box for a HashMap. Stores the Badges for the User in
///     each Channel.
#[derive(Default)]
pub struct States { inner: HashMap<String, Badges> }

impl States {
    /// Get the Badges for the User in a given Channel.
    ///
    /// Input: `&str`
    /// Return: `&str`
    pub fn get(&self, channel: &str) -> &str {
        match self.inner.get(channel) {
            Some(badges) => &badges.output,
            None => BADGE_NONE,
        }
    }

    /// Set the Badges for the User in a given Channel. This is mostly just a
    ///     guarded passthrough to the `HashMap::insert()` of the internal map,
    ///     but with one significant difference: If the current value for the
    ///     given Channel in the Map was created from the same input as has been
    ///     given here, the input is NOT evaluated again.
    /// Returns `true` if the Badges were changed, `false` otherwise.
    ///
    /// Input: `&str`, `&str`, `&str`
    /// Output: `bool`
    pub fn set(&mut self, channel: &str, new: &str, info: &str) -> bool {
        match self.inner.get(channel) {
            Some(old) if new == old.input => false,  // Channel is in Map, with the same Badges.
            _ => {
                self.inner.insert(
                    String::from(channel),
                    Badges::from_str(new, info),
                );
                true
            }
        }
    }
}

safe_static! {
    pub static lazy USERSTATE: RwLock<States> = RwLock::<States>::default();
}
