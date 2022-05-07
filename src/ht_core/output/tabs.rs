use std::{
    collections::hash_map::{Entry, HashMap},
    fmt::{Display, Formatter},
};
use hexchat::{get_channel_name, get_current_channel, get_focused_channel};
use parking_lot::Mutex;


#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TabColor {
    None = 0,
    Event = 1,
    Message = 2,
    Highlight = 3,
}

impl TabColor {
    pub const RESET: Self = Self::None;
}

impl Display for TabColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (*self as u8).fmt(f)
    }
}

//  TODO: Remove this impl after all uses of `echo()` are converted.
impl From<u8> for TabColor {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Event,
            2 => Self::Message,
            3 => Self::Highlight,
            _ => Self::None,
        }
    }
}


/// Tabs: A mapping of HexChat Channel names to their current colors. Provides
///     an interface to change the colors, while also minimizing unnecessary
///     calls to HexChat Commands.
#[derive(Default)]
pub struct Tabs { inner: HashMap<String, TabColor> }

impl Tabs {
    /// Check for the current Channel in the Map of colors. If the Channel is
    ///     not focused AND the provided new color is higher than the current
    ///     one, the Map is updated and the `GUI COLOR` Command is run.
    ///
    /// Input: `TabColor`
    pub fn color(&mut self, color_new: TabColor) {
        if get_focused_channel() != Some(get_current_channel()) {
            match self.inner.entry(get_channel_name()) {
                Entry::Occupied(mut entry) => {
                    let color: &mut TabColor = entry.get_mut();

                    if color_new > *color {
                        //  New color is greater than old color. Replace.
                        *color = color_new;
                        cmd!("GUI COLOR {}", color_new);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(color_new);
                    cmd!("GUI COLOR {}", color_new);
                }
            }
        }
    }

    /// Set the color of the current Channel to 0. Done when a Channel becomes
    ///     focused, so that its unread status is cleared.
    pub fn reset(&mut self) {
        self.inner.insert(get_channel_name(), TabColor::RESET);
        cmd!("GUI COLOR {}", TabColor::RESET);
    }
}


safe_static! {
    pub static lazy TABCOLORS: Mutex<Tabs> = Default::default();
}
