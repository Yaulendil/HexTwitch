use std::{
    collections::hash_map::{Entry, HashMap},
    fmt::{Display, Formatter},
};
use hexchat::get_channel_name;
use parking_lot::RwLock;


const UNK: &str = "Unknown";


#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum PredColor {
    Blue,
    Pink,
    Gray,
}

impl PredColor {
    // pub const fn color(&self) -> &'static str {
    //     match self {
    //         Self::Blue => "blue",
    //         Self::Pink => "pink",
    //         Self::Gray => "gray",
    //     }
    // }

    pub const fn badge(&self, number: usize) -> char {
        let icons = self.icons();
        icons[number % icons.len()]
    }

    const fn icons(&self) -> [char; 2] {
        match self {
            Self::Blue => ['⧮', '⧯'],
            Self::Pink => ['⧰', '⧱'],
            Self::Gray => ['⧲', '⧳'],
        }
    }
}

impl std::str::FromStr for PredColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blue" => Ok(Self::Blue),
            "pink" => Ok(Self::Pink),
            "gray" => Ok(Self::Gray),
            _ => Err(()),
        }
    }
}


#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct PredictionBadge {
    color: PredColor,
    index: u32,
}

impl PredictionBadge {
    pub const fn badge(&self) -> char {
        self.color.badge(self.index as usize)
    }
}

impl std::str::FromStr for PredictionBadge {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (color, index) = s.split_once("-").ok_or(())?;

        Ok(Self {
            color: color.parse()?,
            index: index.parse().map_err(|_| ())?,
        })
    }
}


#[derive(Clone, Default)]
pub struct Predict {
    map: HashMap<PredictionBadge, String>,
}

impl Predict {
    fn pairs(&self) -> Vec<(&PredictionBadge, &String)> {
        self.map.iter().collect()
    }

    fn set_label(&mut self, badge: PredictionBadge, label: &str) -> bool {
        match self.map.get(&badge) {
            Some(s) if label == s => false,
            _ => {
                self.map.insert(badge, label.to_owned());
                true
            }
        }
    }
}

impl Display for Predict {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        macro_rules! args {
            ($p:expr) => {format_args!("{:?} ({})", $p.1, $p.0.badge())};
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
    match variant.parse::<PredictionBadge>() {
        Ok(pb) => {
            let mut map = PREDICT.write();
            let pred: &mut Predict = match map.entry(get_channel_name()) {
                Entry::Vacant(entry) => entry.insert(Default::default()),
                Entry::Occupied(entry) => entry.into_mut(),
            };

            Some(pred.set_label(pb, label))
        }
        Err(_) => None,
    }
}
