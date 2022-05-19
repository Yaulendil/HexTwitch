use std::{collections::HashMap, fmt::{Display, Formatter}};


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


#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
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
        let mut pairs: Vec<BadgeLabel> = self.map.iter()
            .map(BadgeLabel::new)
            .collect();

        pairs.sort_unstable();
        pairs
    }

    pub(super) fn set_label(&mut self, badge: PredictionBadge, label: &str)
        -> bool
    {
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
            [] => f.write_str(UNK),
            [one] => one.fmt(f),
            [one, two] => write!(f, "{} or {}", one, two),
            [most @ .., last] => {
                for each in most {
                    write!(f, "{}, ", each)?;
                }
                write!(f, "or {}", last)
            }
        }
    }
}


#[repr(transparent)]
pub struct MaybePredict<T: std::ops::Deref<Target=Predict>>(pub Option<T>);

impl<T: std::ops::Deref<Target=Predict>> Display for MaybePredict<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(inner) => inner.fmt(f),
            None => f.write_str(UNK),
        }
    }
}
