use hexchat::get_channel_name;
use parking_lot::RwLock;
use std::{
    collections::hash_map::{Entry, HashMap},
    fmt,
};


const INIT: &str = "<UNKNOWN>";


#[derive(Clone)]
pub struct Predict {
    blue: String,
    pink: String,
    gray1: String,
    gray2: String,
}


impl Default for Predict {
    fn default() -> Self {
        Self {
            blue: INIT.into(),
            pink: INIT.into(),
            gray1: INIT.into(),
            gray2: INIT.into(),
        }
    }
}


impl fmt::Display for Predict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "{:?} (⧮), {:?} (⧯), {:?} (⧲), or {:?} (⧳)",
            self.blue,
            self.pink,
            self.gray1,
            self.gray2,
        )
    }
}


safe_static! {
    static lazy PREDICT: RwLock<HashMap<String, Predict>> = Default::default();
}


pub fn get_prediction(channel: &str) -> Option<Predict> {
    PREDICT.read().get(channel).cloned()
}


pub fn update_prediction(variant: &str, label: &str) -> Option<bool> {
    let mut map = PREDICT.write();
    let pred: &mut Predict = match map.entry(get_channel_name()) {
        Entry::Vacant(entry) => entry.insert(Default::default()),
        Entry::Occupied(entry) => entry.into_mut(),
    };

    match variant {
        "blue-1" => {
            if pred.blue != label {
                pred.blue = label.into();
                Some(true)
            } else {
                Some(false)
            }
        }
        "pink-2" => {
            if pred.pink != label {
                pred.pink = label.into();
                Some(true)
            } else {
                Some(false)
            }
        }
        "gray-1" => {
            if pred.gray1 != label {
                pred.gray1 = label.into();
                Some(true)
            } else {
                Some(false)
            }
        }
        "gray-2" => {
            if pred.gray2 != label {
                pred.gray2 = label.into();
                Some(true)
            } else {
                Some(false)
            }
        }
        _ => None,
    }
}
