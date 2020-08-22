use std::collections::HashMap;

use hexchat::{get_channel_name, get_current_channel, get_focused_channel, send_command};
use parking_lot::RwLock;


/// Tabs: A mapping of HexChat Channel names to their current colors. Provides
///     an interface to change the colors, while also minimizing unnecessary
///     calls to HexChat Commands.
#[derive(Default)]
pub struct Tabs { inner: HashMap<String, u8> }

impl Tabs {
    fn new() -> Self { Self::default() }

    /// Check for the current Channel in the Map of colors. If the Channel is
    ///     not focused AND the provided new color is higher than the current
    ///     one, the Map is updated and the `GUI COLOR` Command is run.
    ///
    /// Input: `u8`
    pub fn color(&mut self, color_new: u8) {
        if !get_focused_channel().contains(&get_current_channel()) {
            let name = get_channel_name();

            if &color_new > self.inner.get(&name).unwrap_or(&0) {
                // New color is greater than old color. Replace.
                self.inner.insert(name, color_new);
                send_command(&format!("GUI COLOR {}", color_new));
            }
        }
    }

    /// Set the color of the current Channel to 0. Done when a Channel becomes
    ///     focused, so that its unread status is cleared.
    pub fn reset(&mut self) {
        self.inner.insert(get_channel_name(), 0);
        send_command("GUI COLOR 0");
    }
}


safe_static! {
    pub static lazy TABCOLORS: RwLock<Tabs> = RwLock::new(Tabs::new());
}
