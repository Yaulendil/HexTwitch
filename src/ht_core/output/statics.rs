use std::{collections::{hash_map::{HashMap, Entry}, HashSet}, ops::Deref};
use parking_lot::{
    Mutex,
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
};
use super::{
    Badges,
    tabs::{TabColor, Tabs},
    prediction::{MaybePredict, Predict, PredictionBadge},
    printing::States,
};


safe_static! {
    pub static lazy BADGES_UNKNOWN: BadgesUnknown = Default::default();
    pub static lazy PREDICTIONS: Predictions = Default::default();
    pub static lazy TABCOLORS: TabColors = Default::default();
    pub static lazy USERSTATE: UserState = Default::default();
}


#[derive(Default)]
pub struct BadgesUnknown(RwLock<HashSet<String>>);

impl BadgesUnknown {
    pub fn add(&self, badge: impl Into<String>, default: char) -> char {
        let owned: String = badge.into();

        //  NOTE: Any future checking for global overrides should go here.

        self.0.write().insert(owned);
        default
    }

    pub fn get<'s>(&'s self) -> impl Deref<Target=HashSet<String>> + 's {
        self.0.read()
    }
}


#[derive(Default)]
pub struct Predictions(RwLock<HashMap<String, Predict>>);

impl Predictions {
    pub fn get<'s>(&'s self, channel: &str)
        -> MaybePredict<impl Deref<Target=Predict> + 's>
    {
        let guard = RwLockReadGuard::try_map(
            self.0.read(),
            |map| map.get(channel),
        );

        let inner = match guard {
            Ok(inner) => Some(inner),
            Err(_) => None,
        };

        MaybePredict(inner)
    }

    pub fn update(&self, channel: String, variant: &str, label: &str)
        -> Option<bool>
    {
        match variant.parse::<PredictionBadge>() {
            Ok(pb) => {
                let mut map = self.0.write();
                let pred: &mut Predict = match map.entry(channel) {
                    Entry::Vacant(entry) => entry.insert(Default::default()),
                    Entry::Occupied(entry) => entry.into_mut(),
                };

                Some(pred.set_label(pb, label))
            }
            Err(_) => None,
        }
    }
}


#[derive(Default)]
pub struct TabColors(Mutex<Tabs>);

impl TabColors {
    pub fn color(&self, color_new: TabColor) {
        self.0.lock().color(color_new)
    }

    pub fn reset(&self) {
        self.0.lock().reset()
    }
}


#[derive(Default)]
pub struct UserState(RwLock<States>);

impl UserState {
    pub fn get<'s>(&'s self, channel: &str) -> impl Deref<Target=str> + 's {
        RwLockReadGuard::map(
            self.0.read(),
            |states| states.get(channel),
        )
    }

    pub fn set<'s>(&'s self, channel: String, bstr: String, meta: String)
        -> Option<impl Deref<Target=Badges> + 's>
    {
        RwLockWriteGuard::try_map(
            self.0.write(),
            |states| states.set(channel, bstr, meta),
        ).ok()
    }
}
