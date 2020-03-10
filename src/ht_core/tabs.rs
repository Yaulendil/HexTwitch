use std::collections::HashMap;

use hexchat::{ChannelRef, get_current_channel, print_event_to_channel, PrintEvent};
use parking_lot::RwLock;


pub struct Tabs {
    map: HashMap<String, u8>,
}


impl Tabs {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn reset(&mut self, channel: String) {}

    pub fn color(&mut self, channel: String, color_new: u8) {
        match self.map.get(&channel) {
            Some(color_old) if &color_new <= color_old => {}
            _ => {
                let map = &mut self.map;

                if let Some(b) = map.get_mut(&channel) {
                    *b = color_new;
                } else {
                    map.insert(channel, color_new);
                }
            }
        }
    }
}


safe_static! {
    pub static lazy TABCOLORS: RwLock<Tabs> = RwLock::new(Tabs::new());
}
