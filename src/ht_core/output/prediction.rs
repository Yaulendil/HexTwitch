use hexchat::get_channel_name;
use parking_lot::RwLock;
use std::{
    collections::hash_map::{Entry, HashMap},
    fmt,
};


const ICONS: &[&str] = &["⧮", "⧯", "⧲", "⧳"];
const UNK: &str = "Unknown";


#[derive(Clone, Default)]
pub struct Predict {
    blue: Option<String>,
    pink: Option<String>,
    gray1: Option<String>,
    gray2: Option<String>,
}


impl Predict {
    fn pairs(&self) -> Vec<(&String, &'static str)> {
        let Predict { blue, pink, gray1, gray2 } = self;

        [blue, pink, gray1, gray2]
            .iter()
            .zip(ICONS)
            .filter_map(|(label, &icon)| {
                label.as_ref().map(|inner| (inner, icon))
            })
            .collect()
    }
}


impl fmt::Display for Predict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! args {
            ($pair:expr) => {format_args!("{:?} ({})", $pair.0, $pair.1)};
        }

        match self.pairs().as_slice() {
            &[] => f.write_str(UNK),
            &[one] => f.write_fmt(args!(one)),
            &[one, two] => write!(f, "{} or {}", args!(one), args!(two)),
            &[ref most @ .., last] => {
                for each in most { write!(f, "{}, ", args!(each))?; }
                write!(f, "or {}", args!(last))
            }
        }
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

    macro_rules! update {
        ($field:ident) => {{
            match &pred.$field {
                Some(s) if label == s => Some(false),
                _ => {
                    pred.$field.replace(label.into());
                    Some(true)
                }
            }
        }};
    }

    match variant {
        "blue-1" => update!(blue),
        "pink-2" => update!(pink),
        "gray-1" => update!(gray1),
        "gray-2" => update!(gray2),
        _ => None,
    }
}
