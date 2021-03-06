use std::{
    collections::{hash_map::{Entry, HashMap}, HashSet},
    ops::{Deref, DerefMut},
};
use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use super::{
    Badges,
    channels::ChannelData,
    tabs::{TabColor, Tabs},
    printing::States,
};


safe_static! {
    pub static lazy BADGES_UNKNOWN: BadgesUnknown = Default::default();
    pub static lazy CHANNELS: Channels = Default::default();
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
pub struct Channels(RwLock<HashMap<String, ChannelData>>);

impl Channels {
    pub fn current(&self) -> impl DerefMut<Target=ChannelData> + '_ {
        self.ensure(hexchat::get_channel_name())
    }

    pub fn ensure<'s>(&'s self, channel: String)
        -> impl DerefMut<Target=ChannelData> + 's
    {
        RwLockWriteGuard::map(
            self.0.write(),
            |map| match map.entry(channel) {
                Entry::Occupied(e) => e.into_mut(),
                Entry::Vacant(e) => e.insert(Default::default()),
            }
        )
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

    pub fn has(&self, channel: &str) -> bool {
        self.0.read().has(channel)
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
