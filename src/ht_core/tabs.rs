use std::collections::HashMap;

use hexchat::{ChannelRef, get_channel_name, get_focused_channel, send_command};
use parking_lot::RwLock;


pub fn is_focused(channel: ChannelRef) -> bool {
    channel == get_focused_channel().unwrap()
}


pub struct Tabs {
    map: HashMap<String, u8>,
}

impl Tabs {
    fn new() -> Self { Self { map: HashMap::new() } }

    pub fn color(&mut self, channel: ChannelRef, color_new: u8) {
        let name = get_channel_name();

        match self.map.get(&name) {
            Some(color_old) if &color_new <= color_old => {}
            _ if is_focused(channel) => {}
            _ => {
                let map = &mut self.map;

                if let Some(b) = map.get_mut(&name) {
                    *b = color_new;
                } else {
                    map.insert(name, color_new);
                }

                send_command(&format!("GUI COLOR {}", color_new));
            }
        }
    }

    pub fn reset(&mut self) {
        let name = get_channel_name();
        let map = &mut self.map;

        if let Some(b) = map.get_mut(&name) {
            *b = 0;
        } else {
            map.insert(name, 0);
        }

        send_command("GUI COLOR 0");
    }
}


safe_static! {
    pub static lazy TABCOLORS: RwLock<Tabs> = RwLock::new(Tabs::new());
}
