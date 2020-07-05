use std::collections::HashMap;

use hexchat::{ChannelRef, get_channel_name, get_focused_channel, send_command};
use parking_lot::RwLock;


fn is_focused(channel: ChannelRef) -> bool {
    Some(channel) == get_focused_channel()
}


/// Tabs: A mapping of HexChat Channel names to their current colors. Provides
///     an interface to change the colors, while also minimizing unnecessary
///     calls to HexChat Commands.
pub struct Tabs(HashMap<String, u8>);

impl Tabs {
    fn new() -> Self { Self(HashMap::new()) }

    /// Check the provided `ChannelRef` in the Map of colors. If the Channel is
    ///     not focused AND the provided new color is higher than the current
    ///     one, the Map is updated and the `GUI COLOR` Command is run.
    ///
    /// Input: `ChannelRef`, `u8`
    pub fn color(&mut self, channel: ChannelRef, color_new: u8) {
        if !is_focused(channel) {
            let name = get_channel_name();

            match self.0.get_mut(&name) {
                Some(color_old) if &color_new <= color_old => {}  // No change.
                Some(color_old) => {
                    // New color is greater than old color. Replace.
                    *color_old = color_new;
                    send_command(&format!("GUI COLOR {}", color_new));
                }
                None => {
                    // No old color. Insert new color.
                    self.0.insert(name, color_new);
                    send_command(&format!("GUI COLOR {}", color_new));
                }
            }
        }
    }

    /// Set the color of the current Channel to 0. Done when a Channel becomes
    ///     focused, so that its unread status is cleared.
    pub fn reset(&mut self) {
        let name = get_channel_name();

        match self.0.get_mut(&name) {
            Some(color) => { *color = 0; }
            None => { self.0.insert(name, 0); }
        }

        send_command("GUI COLOR 0");
    }
}


safe_static! {
    pub static lazy TABCOLORS: RwLock<Tabs> = RwLock::new(Tabs::new());
}
