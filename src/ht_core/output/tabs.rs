use std::collections::HashMap;

use hexchat::{ChannelRef, get_channel_name, get_focused_channel, send_command};
use parking_lot::RwLock;


fn is_focused(channel: ChannelRef) -> bool {
    channel == get_focused_channel().unwrap()
}


pub struct Tabs(HashMap<String, u8>);

impl Tabs {
    fn new() -> Self { Self(HashMap::new()) }

    pub fn color(&mut self, channel: ChannelRef, color_new: u8) {
        if !is_focused(channel) {
            let name = get_channel_name();

            match self.0.get_mut(&name) {
                Some(color_old) if &color_new <= color_old => {}
                guard => {
                    if let Some(b) = guard {
                        *b = color_new;
                    } else {
                        self.0.insert(name, color_new);
                    }

                    send_command(&format!("GUI COLOR {}", color_new));
                }
            }
        }
    }

    pub fn reset(&mut self) {
        let name = get_channel_name();

        if let Some(b) = self.0.get_mut(&name) {
            *b = 0;
        } else {
            self.0.insert(name, 0);
        }

        send_command("GUI COLOR 0");
    }
}


safe_static! {
    pub static lazy TABCOLORS: RwLock<Tabs> = RwLock::new(Tabs::new());
}
