use std::collections::hash_map::{Entry, HashMap};
use cached::proc_macro::cached;
use hexchat::{print_event, PrintEvent};
use crate::irc::split_at_char;
use super::{
    prediction::PredictionBadge,
    statics::{BADGES_UNKNOWN, CHANNELS, TABCOLORS},
    tabs::TabColor,
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
/// Input: `PrintEvent`, `&[impl AsRef<str>]`, `TabColor`
#[inline]
pub fn echo(event: PrintEvent, args: &[impl AsRef<str>], tab_color: TabColor) {
    print_event(event, args);
    TABCOLORS.color(tab_color);
}


pub fn alert_basic(message: impl AsRef<str>) {
    echo(EVENT_NORMAL, &[message], TabColor::Event);
}


pub fn alert_error(message: impl AsRef<str>) {
    echo(EVENT_ERR, &[message], TabColor::Event);
}


pub fn alert_subscription(message: impl AsRef<str>) {
    echo(EVENT_ALERT, &["SUBSCRIPTION", message.as_ref()], TabColor::Message);
}


pub fn alert_sub_upgrade(message: impl AsRef<str>) {
    echo(EVENT_ALERT, &["UPGRADE", message.as_ref()], TabColor::Message);
}


pub fn change_topic(channel: &str, new: &str) {
    if let Some(cref) = hexchat::get_channel(crate::NETWORK, channel) {
        let topic_old = hexchat::get_topic(&cref);
        let print_old: Option<&String>;
        let apply_new: bool;

        match &topic_old {
            Some(old) if new != old => {
                //  New topic is DIFFERENT from the old topic. Print the old
                //      one, and apply the new one.
                print_old = Some(old);
                apply_new = true;
            }
            Some(_) => {
                //  New topic is the SAME as the old topic. Do not print the old
                //      one, and do not apply the new one.
                print_old = None;
                apply_new = false;
            }
            None => {
                //  No topic is set. Apply the new one.
                print_old = None;
                apply_new = true;
            }
        };

        if let Some(old) = print_old {
            hexchat::print_event_to_channel(
                &cref,
                PrintEvent::TOPIC,
                &[channel, old],
                // TabColor::None,
            );
        }

        if apply_new {
            cmd!("RECV :Twitch!twitch@twitch.tv TOPIC {} :{}", channel, new);
        }
    }
}


/*pub fn print_topic(channel: &str) -> Option<()> {
    if let Some(cref) = hexchat::get_channel(crate::NETWORK, channel) {
        let topic: String = hexchat::get_topic(&cref)?;

        if !topic.is_empty() {
            hexchat::print_event_to_channel(
                &cref,
                PrintEvent::TOPIC,
                &[channel, &topic],
                // TabColor::None,
            );
        }
    }

    Some(())
}*/


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
        "artist-badge"      /**/ => 'α',

        "sub-gift-leader"   /**/ => '⁘',
        // "sub-gifter"        /**/ => highest(rank.parse().unwrap_or(0), &GIFTS),
        "sub-gifter"        /**/ => ':',
        "bits-charity"      /**/ => '🝔',
        "bits-leader"       /**/ => '❖',
        "bits"              /**/ => highest(rank.parse().unwrap_or(0), &BITS),
        "hype-train"        /**/ => '.',
        // "moments"           /**/ => highest(rank.parse().unwrap_or(0), &MOMENT),
        "moments"           /**/ => 'm',

        "partner"           /**/ => '✓',
        "turbo"             /**/ => '+',
        "premium"           /**/ => '±',
        "no_audio"          /**/ => 'Ⓐ', // Alternatively '🔇'?
        "no_video"          /**/ => 'Ⓥ', // Alternatively '👁'?

        "glhf-pledge"       /**/ => '~',
        "anonymous-cheerer" /**/ => '*',

        "ambassador"        /**/ => 'a',
        "predictions"       /**/ => prediction_badge(rank),

        s if s.starts_with("glitchcon") => 'g',
        s if s.starts_with("twitchcon") => 'c',
        s if s.starts_with("overwatch-league-insider") => 'w',
        s if is_game_badge(s) => 'G',
        s => BADGES_UNKNOWN.add(s, '?'),
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
    //  twitch why
    match pred.parse::<PredictionBadge>() {
        Ok(pb) => pb.badge(),
        Err(_) => BADGES_UNKNOWN.add(format!("predictions/{}", pred), 'p'),
    }
}


const ICONS_OP: &[char] = &['🜲', '🜨', '⛨', '🗡'];


/// Badges: A Struct storing the Input and Output of the process of breaking
///     down a badge value. This effectively serves the purpose of a Cached
///     Function.
#[derive(Clone, Default)]
pub struct Badges {
    badges: String,
    badge_info: String,
    pub output: Option<String>,
    // mode_op: bool,
    // mode_voiced: bool,
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
    fn from_str(badges: String, badge_info: String) -> Self {
        const SUB: &str = "subscriber";
        const KEY: &str = "subscriber/";
        const LEN: usize = KEY.len();

        let output: Option<String> = if !badges.is_empty() {
            let mut output: String = String::with_capacity(16);
            let check_subs: bool = !badge_info.is_empty();

            for pair_badge in badges.split(',') {
                //  Twitch now provides the number of months attached to a Sub
                //      Badge separately, in the `badge-info` Tag. The number
                //      attached directly to the Badge itself will only reflect
                //      the correct number of months if the channel has a custom
                //      icon set for the tier.
                if check_subs && pair_badge.starts_with(KEY) {
                    //  This is a special case, because we do not actually care
                    //      about the rank of the Subscriber Badge. What we want
                    //      to use as the rank is in the `badge-info` Tag.
                    for pair_info in badge_info.split(',') {
                        if pair_info.starts_with(KEY) {
                            output.push(get_badge(SUB, &pair_info[LEN..]));
                            break;
                        }
                    }
                } else {
                    let (class, rank) = split_at_char(pair_badge, '/');
                    output.push(get_badge(class, rank));
                }
            }

            if !output.is_empty() {
                output.push(' ');
                Some(output)
            } else {
                None
            }
        } else {
            None
        };

        Self { badges, badge_info, output }
    }

    pub fn as_str(&self) -> &str {
        match &self.output {
            Some(oput) => oput,
            None => "",
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.output.is_none()
    }

    #[allow(dead_code)]
    pub fn is_op(&self) -> bool {
        match &self.output {
            Some(s) => s.contains(ICONS_OP),
            None => false,
        }
    }

    // #[allow(dead_code)]
    // pub const fn is_voiced(&self) -> bool {
    //     self.mode_voiced
    // }

    /// Update the map of Predictions to include the data in the message used to
    ///     create these badges.
    pub fn update_prediction(&self, channel: &str) -> bool {
        const KEY: &str = "predictions/";
        const LEN: usize = KEY.len();

        fn is_prediction(pair: &&str) -> bool { pair.starts_with(KEY) }

        if !self.badges.is_empty() && !self.badge_info.is_empty() {
            //  Search the badge info tag first; It is likely much shorter than
            //      the badges tag, so if there is no tag found for Predictions,
            //      this order will fail and move on more quickly.
            if let Some(info) = self.badge_info.split(',').find(is_prediction) {
                if let Some(rank) = self.badges.split(',').find(is_prediction) {
                    let variant = &rank[LEN..];
                    let label = &info[LEN..];

                    if let Ok(badge) = variant.parse::<PredictionBadge>() {
                        let mut cref = CHANNELS.ensure(channel.to_owned());
                        let predict = &mut cref.predictions;
                        let update = predict.set_label(badge, label);

                        if let Some(mode) = update.new_mode() {
                            alert_basic(&format!(
                                "Prediction type changed to {}.",
                                mode,
                            ));
                        }

                        if update.changed_label() {
                            alert_basic(&format!(
                                "Prediction labels updated: {}",
                                predict,
                            ));
                        }

                        return update.changed_either();
                    }
                }
            }
        }

        false
    }
}

impl AsRef<str> for Badges {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<T> AsRef<T> for Badges where
    str: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.as_str().as_ref()
    }
}


/// Passthrough function required for caching.
#[cached(size = 50)]
pub fn badge_parse(badges: String, badge_info: String) -> Badges {
    Badges::from_str(badges, badge_info)
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
            .map(Badges::as_str)
            .unwrap_or(Badges::NONE)
    }

    pub fn has(&self, channel: &str) -> bool {
        self.inner.contains_key(channel)
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
        -> Option<&mut Badges>
    {
        match self.inner.entry(channel) {
            Entry::Vacant(entry) => Some(entry.insert(badge_parse(bstr, meta))),
            Entry::Occupied(entry) => {
                let badges: &mut Badges = entry.into_mut();

                if badges.badges != bstr || badges.badge_info != meta {
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
