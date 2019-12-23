use std::collections::HashMap;

use hexchat_plugin::{Context, PluginHandle};

use super::ircv3::{Message, split_at_first};


pub struct Badges {
    input: String,
    output: String,
}


impl Badges {
    pub fn new(input: &str) -> Badges {
        let mut i: i32 = 0;
        let mut output: String = String::new();

        for pair in input.split(",") {
            let (class, _rank) = split_at_first(pair, "/");

            //  TODO: Do not hardcode this.
            match match class {
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
                None => {}
                Some(c) => {
                    i += 1;
                    output.push(c);
                }
            }

            if i >= 3 { break; }
        }

        Badges {
            input: input.to_string(),
            output,
        }
    }
}


pub struct States {
    map: Option<HashMap<String, Badges>>,
}


impl States {
    fn ensure(&mut self) {
        match &self.map {
            None => {
                self.map.replace(HashMap::new()).unwrap();
            }
            Some(_) => {}
        };
    }

    pub fn get(&mut self, channel: &str) -> &Badges {
        self.ensure();

        self.map.as_ref().unwrap().get(channel).unwrap()
    }

    pub fn set(&mut self, channel: String, new: &str) {
        let old: &Badges = self.get(&channel);

        if new != &old.input {
            let mut new_state: Badges = Badges::new(new);

            self.map.as_mut().unwrap().get_mut(&channel).replace(&mut new_state);
        }
    }
}


pub static mut USERSTATE: States = States {
    map: None,
};


pub fn print(ph: &mut PluginHandle, ctx: Context, msg: Message) {
    ;
}
