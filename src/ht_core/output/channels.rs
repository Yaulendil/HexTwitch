use std::ops::{Deref, Neg};
use hexchat::print_plain;
use super::prediction::*;


#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Change<T> {
    pub old: T,
    pub new: T,
}

impl<T: Copy> Change<T> {
    pub fn new(target: &mut T, new: T) -> Self {
        let old = *target;
        *target = new;

        Self { old, new }
    }
}

impl<T> Deref for Change<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.new
    }
}

impl<T> Neg for Change<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let Self { old, new } = self;
        Self {
            old: new,
            new: old,
        }
    }
}


fn change<T: Copy>(target: &mut T, new: T) -> Change<T> {
    Change::new(target, new)
}


#[derive(Clone, Copy, Debug)]
pub enum FollowMode {
    /// Anyone can chat.
    Off,
    /// Must follow to chat.
    FollowAny,
    /// Must follow for a number of minutes to chat.
    ForMinutes(usize),
}

impl Default for FollowMode {
    fn default() -> Self { Self::Off }
}


pub enum StateChange {
    Slow(Change<Option<usize>>),
    Unique(Change<bool>),
    Emotes(Change<bool>),
    Followers(Change<FollowMode>),
    Subscribers(Change<bool>),
    Rituals(Change<Option<usize>>),
    RoomId(Change<Option<usize>>),
}


#[derive(Clone, Copy, Debug, Default)]
pub struct RoomState {
    pub slow: Option<usize>,
    pub unique: bool,
    pub emotes: bool,
    pub followers: FollowMode,
    pub subscribers: bool,

    pub rituals: Option<usize>,
    pub room_id: Option<usize>,
}

impl RoomState {
    pub fn update<'k>(
        &mut self,
        key: &'k str,
        value: &str,
    ) -> Result<StateChange, &'k str> {
        match key {
            "emote-only" => {
                let new = value != "0";
                Ok(StateChange::Emotes(change(&mut self.emotes, new)))
            }
            "r9k" => {
                let new = value != "0";
                Ok(StateChange::Unique(change(&mut self.unique, new)))
            }
            "subs-only" => {
                let new = value != "0";
                Ok(StateChange::Subscribers(change(&mut self.subscribers, new)))
            }
            "slow" => {
                let new = match value.parse::<usize>().unwrap_or(0) {
                    0 => None,
                    n => Some(n),
                };
                Ok(StateChange::Slow(change(&mut self.slow, new)))
            }
            "followers-only" => {
                let new = match value.parse::<isize>().unwrap_or(-1) {
                    //  No follow requirement.
                    n if n < 0 => FollowMode::Off,
                    //  Must follow to talk.
                    0 => FollowMode::FollowAny,
                    //  Must follow for N minutes before talking.
                    n => FollowMode::ForMinutes(n as usize),
                };
                Ok(StateChange::Followers(change(&mut self.followers, new)))
            }
            "rituals" => {
                let new = value.parse().ok();
                Ok(StateChange::Rituals(change(&mut self.rituals, new)))
            }
            "room-id" => {
                let new = value.parse().ok();
                Ok(StateChange::RoomId(change(&mut self.room_id, new)))
            }
            key => Err(key),
        }
    }
}

impl RoomState {
    pub fn report_id(&self) {
        match self.room_id {
            Some(id) => print_plain(&format!("Room ID is set to {}.", id)),
            None => print_plain("Room ID is not set."),
        }
    }

    pub fn report_rituals(&self) {
        match self.rituals {
            Some(n) => print_plain(&format!("Ritual value is set to {}.", n)),
            None => print_plain("Ritual value is disabled."),
        }
    }

    pub fn report_slow(&self) {
        match self.slow {
            Some(sec) => print_plain(&format!("Slow mode is set to {}s.", sec)),
            None => print_plain("Slow mode is disabled."),
        }
    }

    pub fn report_followers(&self) {
        match self.followers {
            FollowMode::Off => print_plain("Followers-Only mode is disabled."),
            FollowMode::FollowAny => print_plain("Followers-Only mode is enabled."),
            FollowMode::ForMinutes(min) => print_plain(
                &format!("{}-minute Followers-Only mode is enabled.", min)
            ),
        }
    }

    pub fn report_unique(&self) {
        match self.unique {
            true => print_plain("Unique mode is enabled."),
            false => print_plain("Unique mode is NOT enabled."),
        }
    }

    pub fn report_emotes(&self) {
        match self.emotes {
            true => print_plain("Emote Only mode is enabled."),
            false => print_plain("Emote Only mode is NOT enabled."),
        }
    }

    pub fn report_subscribers(&self) {
        match self.subscribers {
            true => print_plain("Subscribers-Only mode is enabled."),
            false => print_plain("Subscribers-Only mode is NOT enabled."),
        }
    }
}


#[derive(Debug, Default)]
pub struct ChannelData {
    pub predictions: Predict,
    pub roomstate: RoomState,
}
