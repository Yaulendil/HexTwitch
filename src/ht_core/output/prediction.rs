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
    pub const fn color(&self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Pink => "pink",
            Self::Gray => "gray",
        }
    }

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

impl Display for PredictionBadge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.color.color(), self.index)
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


#[derive(Clone, Copy)]
struct BadgeLabel<'s> {
    badge: &'s PredictionBadge,
    label: &'s String,
}

impl<'s> BadgeLabel<'s> {
    const fn new(pair: (&'s PredictionBadge, &'s String)) -> Self {
        let (badge, label) = pair;

        Self { badge, label }
    }
}

impl<'s> Display for BadgeLabel<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            // "\"{name}\"/'{icon}' ({text:?})",
            "{text:?} ({icon}: {name})",
            icon = self.badge.badge(),
            name = self.badge,
            text = self.label,
        )
    }
}


#[derive(Clone, Default)]
pub struct Predict {
    map: HashMap<PredictionBadge, String>,
}

impl Predict {
    fn pairs(&self) -> Vec<BadgeLabel> {
        self.map.iter().map(BadgeLabel::new).collect()
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
        match self.pairs().as_slice() {
            &[] => f.write_str(UNK),
            &[one] => one.fmt(f),
            &[one, two] => write!(f, "{} or {}", one, two),
            &[ref most @ .., last] => {
                for each in most {
                    write!(f, "{}, ", each)?;
                }
                write!(f, "or {}", last)
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
