use std::collections::HashMap;

use hexchat::{get_current_channel, print_event_to_channel, PrintEvent};
use parking_lot::RwLock;

use super::ircv3::split_at_first;


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


pub fn echo(event: PrintEvent, args: &[impl AsRef<str>]) {
    print_event_to_channel(&get_current_channel(), event, args);
}


pub static WHISPER_SIDES: &str = "==";


safe_static! {
    static lazy BADGE_NONE: &str = "";
}
const MAX_BADGES: usize = 3;


/// Badges: A Struct storing the Input and Output of the process of breaking
///     down a badge value. This effectively serves the purpose of a Cached
///     Function.
pub struct Badges {
    input: String,
    pub output: String,
}

impl Badges {
    /// Break down a string to find the final set of characters. The original
    ///     will be stored.
    ///
    /// Input: `&str`
    /// Return: `Badges`
    pub fn new(input: &str) -> Self {
        let mut i: usize = 0;
        let mut output: String = String::new();

        for pair in input.split(",") {
            if i >= MAX_BADGES { break; }

            let (class, _rank) = split_at_first(pair, "/");

            //  TODO: Do not hardcode this.
            if let Some(c) = match class {
                "broadcaster" /**/ => Some('🜲'),
                "staff"       /**/ => Some('⚙'),
                "admin"       /**/ => Some('α'),
                "global-mod"  /**/ => Some('μ'),
                "moderator"   /**/ => Some('🗡'),
                "subscriber"  /**/ => None,
                "vip"         /**/ => Some('⚑'),
                "sub-gifter"  /**/ => Some(':'),
                "bits-leader" /**/ => Some('❖'),
                "bits"        /**/ => None,
                "partner"     /**/ => Some('✓'),
                "turbo"       /**/ => Some('+'),
                "premium"     /**/ => Some('±'),
                _ => None,
            } {
                i += 1;
                output.push(c);
            }
        }

        Self {
            input: input.to_string(),
            output,
        }
    }
}


/// States: Effectively a Box for a HashMap. Stores the Badges for the User in
///     each Channel.
pub struct States {
    map: Option<HashMap<String, Badges>>,
}

impl States {
    pub fn init(&mut self) {
        if self.map.is_none() {
            self.map.replace(HashMap::new());
        }
    }

    /// Get the Badges for the User in a given Channel.
    ///
    /// Input: `&str`
    /// Return: `&str`
    pub fn get(&self, channel: &str) -> &str {
        match self.map.as_ref().unwrap().get(channel) {
            Some(badges) => badges.output.as_str(),
            None => "",
        }
    }

    /// Set the Badges for the User in a given Channel. This is mostly just a
    ///     guarded passthrough to the `HashMap::insert()` of the internal map,
    ///     but with one significant difference: If the current value for the
    ///     given Channel in the Map was created from the same input as has been
    ///     given here, the input is NOT evaluated again.
    ///
    /// Input: `String`, `&str`
    pub fn set(&mut self, channel: String, new: &str) {
        match self.map.as_ref().unwrap().get(&channel) {
            Some(old) if new == old.input => {}  // Channel is in Map, with the same Badges.
            _ => {
                let badges = Badges::new(new);
                let map = self.map.as_mut().unwrap();

                if let Some(b) = map.get_mut(&channel) {
                    *b = badges;
                } else {
                    map.insert(channel, badges);
                }
            }
        }
    }
}

safe_static! {
    pub static lazy USERSTATE: RwLock<States> = RwLock::new(States { map: None });
}
