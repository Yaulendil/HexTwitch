use std::{collections::HashMap, fmt::{Display, Formatter}};


const UNK: &str = "Unknown";

const I_BLUE: [char; 10] = ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾', '❿'];
const I_PINK: [char; 2] = ['❶', '❷'];
const I_GRAY: [char; 2] = ['⧲', '⧳'];

const LABEL_BLUE: &str = "blue";
const LABEL_PINK: &str = "pink";
const LABEL_GRAY: &str = "gray";


#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum PredictionBadge {
    Blue(u32),
    Pink(u32),
    Gray(u32),
}

impl PredictionBadge {
    pub const fn badge(&self) -> char {
        let icons: &[char];
        let value: usize;

        match self {
            Self::Blue(n) => {
                icons = &I_BLUE;
                value = *n as _;
            }
            Self::Pink(n) => {
                icons = &I_PINK;
                value = *n as _;
            }
            Self::Gray(n) => {
                icons = &I_GRAY;
                value = *n as _;
            }
        }

        icons[value.saturating_sub(1) % icons.len()]
    }
}

/// Basic properties.
#[allow(dead_code)]
impl PredictionBadge {
    pub const fn is_blue(&self) -> bool {
        matches!(self, Self::Blue(_))
    }

    pub const fn is_pink(&self) -> bool {
        matches!(self, Self::Pink(_))
    }

    pub const fn is_gray(&self) -> bool {
        matches!(self, Self::Gray(_))
    }
}

/// Higher-level deductions about possible states.
#[allow(dead_code)]
impl PredictionBadge {
    const fn can_be_blue_pink(&self) -> bool {
        match self {
            Self::Blue(1) => true,
            Self::Blue(_) => false,
            Self::Pink(_) => true,
            Self::Gray(_) => false,
        }
    }

    const fn must_be_blue10(&self) -> bool {
        match self {
            Self::Blue(1) => false,
            Self::Blue(_) => true,
            Self::Pink(_) => false,
            Self::Gray(_) => false,
        }
    }

    const fn likely_mode(&self) -> Option<PredictMode> {
        match self {
            //  Blue 1. Maybe 10 blues, or maybe blue/pink.
            Self::Blue(1) => None,

            //  Pink 2. Most likely blue/pink. If they add another mode using
            //      pinks, this will need to be changed. Probably with a very
            //      extensive rework of the whole thing. Again.
            Self::Pink(2) => Some(PredictMode::BluePink),

            //  Blue higher than 1.
            Self::Blue(_) => Some(PredictMode::Blue10),

            //  Pink, but not 2. Does not fit into any currently known mode.
            Self::Pink(_) => None,

            //  Grays do not seem to be mixed with other colors.
            Self::Gray(_) => Some(PredictMode::Gray2),
        }
    }

    const fn guess_mode(&self) -> PredictMode {
        match self {
            Self::Blue(_) => PredictMode::Blue10,
            Self::Pink(_) => PredictMode::BluePink,
            Self::Gray(_) => PredictMode::Gray2,
        }
    }
}

impl Display for PredictionBadge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let color: &'static str;
        let value: u32;

        match self {
            Self::Blue(n) => {
                color = LABEL_BLUE;
                value = *n;
            }
            Self::Pink(n) => {
                color = LABEL_PINK;
                value = *n;
            }
            Self::Gray(n) => {
                color = LABEL_GRAY;
                value = *n;
            }
        }

        write!(f, "{}-{}", color, value)
    }
}

impl std::str::FromStr for PredictionBadge {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (color, value) = s.split_once("-").ok_or(())?;
        let n = value.parse().or(Err(()))?;

        match color {
            LABEL_BLUE => Ok(Self::Blue(n)),
            LABEL_PINK => Ok(Self::Pink(n)),
            LABEL_GRAY => Ok(Self::Gray(n)),
            _ => Err(()),
        }
    }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictMode {
    /// Up to ten possible outcomes, all of them blue.
    Blue10,
    /// One blue outcome, and one pink outcome.
    BluePink,
    /// Two gray outcomes. It is not clear when this would show up, or whether
    ///     it is even actively used.
    Gray2,
    /// An unknown mode. These rules are undefined.
    Unknown,
}

impl PredictMode {
    pub const fn can_include(&self, badge: &PredictionBadge) -> bool {
        match self {
            Self::Blue10 => badge.is_blue(),
            Self::BluePink => badge.can_be_blue_pink(),
            Self::Gray2 => badge.is_gray(),
            Self::Unknown => true,
        }
    }

    pub const fn desc(&self) -> &'static str {
        match self {
            Self::Blue10 => "Blue 1 through Blue 10",
            Self::BluePink => "Blue vs Pink",
            Self::Gray2 => "Gray 1 vs Gray 2",
            Self::Unknown => "an unknown mode",
        }
    }
}

impl Default for PredictMode {
    fn default() -> Self { Self::Blue10 }
}

impl Display for PredictMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.desc().fmt(f)
    }
}


pub enum PredictUpdate {
    ChangedNone,
    ChangedLabel,
    ChangedMode(PredictMode),
    ChangedBoth(PredictMode),
}

#[allow(dead_code)]
impl PredictUpdate {
    pub const fn changed_either(&self) -> bool {
        !matches!(self, Self::ChangedNone)
    }

    pub const fn changed_label(&self) -> bool {
        matches!(self, Self::ChangedLabel | Self::ChangedBoth(_))
    }

    pub const fn changed_mode(&self) -> bool {
        matches!(self, Self::ChangedMode(_) | Self::ChangedBoth(_))
    }

    pub const fn changed_both(&self) -> bool {
        matches!(self, Self::ChangedBoth(_))
    }

    pub const fn new_mode(&self) -> Option<PredictMode> {
        match self {
            Self::ChangedMode(mode) => Some(*mode),
            Self::ChangedBoth(mode) => Some(*mode),
            _ => None,
        }
    }
}


#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct BadgeLabel<'s> {
    badge: &'s PredictionBadge,
    label: &'s String,
}

impl<'s> BadgeLabel<'s> {
    const fn from_pair(pair: (&'s PredictionBadge, &'s String)) -> Self {
        let (badge, label) = pair;

        Self { badge, label }
    }
}

impl<'s> Display for BadgeLabel<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            // "\"{name}\"/'{icon}' ({text:?})",
            // "{text:?} ({icon}: {name})",
            "{icon}: {text:?}",
            icon = self.badge.badge(),
            // name = self.badge,
            text = self.label,
        )
    }
}


//  At the time of this writing, it seems that a prediction may be between
//      `blue-1` and `pink-2`, or between `blue-1`, `blue-2`, (...), and
//      `blue-10`.
#[derive(Clone, Debug, Default)]
pub struct Predict {
    map: HashMap<PredictionBadge, String>,
    mode: PredictMode,
}

impl Predict {
    pub const fn mode(&self) -> PredictMode { self.mode }

    fn pairs(&self) -> Vec<BadgeLabel> {
        let mut pairs: Vec<BadgeLabel> = self.map.iter()
            .filter(|(badge, _)| self.mode.can_include(badge))
            .map(BadgeLabel::from_pair)
            .collect();

        pairs.sort_unstable();
        pairs
    }

    #[allow(dead_code)]
    fn pairs_all(&self) -> Vec<BadgeLabel> {
        let mut pairs: Vec<BadgeLabel> = self.map.iter()
            .map(BadgeLabel::from_pair)
            .collect();

        pairs.sort_unstable();
        pairs
    }

    pub(super) fn set_label(&mut self, badge: PredictionBadge, label: &str)
        -> PredictUpdate
    {
        let first: bool = self.map.is_empty();
        let changed_mode: bool = self.switch_mode(&badge) || first;
        let changed_label: bool = match self.map.get(&badge) {
            Some(s) if label == s => false,
            _ => {
                self.map.insert(badge, label.to_owned());
                true
            }
        };

        match (changed_mode, changed_label) {
            (false, false) => PredictUpdate::ChangedNone,
            (false, true) => PredictUpdate::ChangedLabel,
            (true, false) => PredictUpdate::ChangedMode(self.mode),
            (true, true) => PredictUpdate::ChangedBoth(self.mode),
        }
    }

    fn switch_mode(&mut self, badge: &PredictionBadge) -> bool {
        use crate::prefs::{HexPrefGet, PREF_DEBUG};

        if self.mode.can_include(badge) {
            //  No need to change.
            false
        } else if let Some(mode) = badge.likely_mode() {
            //  Need to change mode.
            self.mode = mode;
            true
        } else if PREF_DEBUG.get() == Some(true) {
            //  Need to change, but no idea to what. Debug enabled, so set to
            //      unknown. This way all values will be printed.
            self.mode = PredictMode::Unknown;
            true
        } else {
            //  Need to change, but no idea to what.
            false
        // } else {
        //     //  Need to change, but do not know to what. Make a guess.
        //     let mode = badge.guess_mode();
        //
        //     if self.mode != mode {
        //         self.mode = mode;
        //         true
        //     } else {
        //         false
        //     }
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
