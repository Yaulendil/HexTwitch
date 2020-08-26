mod events;
mod irc;
mod output;


use chrono::{DateTime, Utc};
use hexchat::{
    ChannelRef,
    delete_pref,
    EatMode,
    get_channel_name,
    get_network_name,
    get_pref_int,
    get_pref_string,
    get_prefs,
    PrintEvent,
    send_command,
    set_pref_int,
    set_pref_string,
    strip_formatting,
};
use parking_lot::Mutex;

use irc::Message;
use output::{
    echo,
    EVENT_ERR,
    EVENT_NORMAL,
    print_with_irc,
    print_without_irc,
    TABCOLORS,
};


#[derive(Default)]
struct Sponge {
    signature: Option<String>,
    value: Option<Message>,
}

impl Sponge {
    /// Place a Message into the Sponge. The previous Message in the Sponge, if
    ///     any, is removed. Takes Ownership of the new Message.
    ///
    /// Input: `Message`
    fn put(&mut self, new: Message) {
        self.signature.replace(new.get_signature());
        self.value.replace(new);
    }

    /// Remove the Message from the Sponge, but only if the current Message has
    ///     the Signature specified.
    ///
    /// Input: `&str`
    /// Return: `Option<Message>`
    fn pop(&mut self, signature: &str) -> Option<Message> {
        match &self.signature {
            Some(sig) if sig == signature => self.value.take(),
            _ => None,
        }
    }
}


safe_static! {
    static lazy CURRENT: Mutex<Sponge> = Mutex::new(Sponge::default());
}


fn check_message(channel: &str, author: &str) -> Option<Message> {
    let sig: &str = &format!(
        "{}:{}",
        &channel,
        strip_formatting(author).unwrap_or_default(),
    );

    CURRENT.lock().pop(sig)
}


/// Reset the Color of a newly-focused Tab.
pub(crate) fn cb_focus(_channel: ChannelRef) -> EatMode {
    if get_network_name().unwrap_or_default().eq_ignore_ascii_case("twitch") {
        TABCOLORS.write().reset();
    }
    EatMode::None
}


/// Hide a Join Event if it is fake.
pub(crate) fn cb_join(_etype: PrintEvent, word: &[String]) -> EatMode {
    if get_network_name().unwrap_or_default().eq_ignore_ascii_case("twitch")
        && !word[2].contains("tmi.twitch.tv")
    {
        EatMode::All
    } else {
        EatMode::None
    }
}


pub(crate) fn cb_print(etype: PrintEvent, word: &[String]) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let channel = get_channel_name();

            if let Some(msg) = check_message(&channel, &word[0]) {
                //  Message comes from Server. IRC Representation available.
                print_with_irc(&channel, etype, word, msg)
            } else if etype == PrintEvent::YOUR_MESSAGE
                || etype == PrintEvent::YOUR_ACTION {
                //  No IRC Representation available for Message.
                print_without_irc(&channel, etype, word)
            } else {
                EatMode::None
            }
        }
        _ => EatMode::None
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub(crate) fn cb_server(word: &[String], _dt: DateTime<Utc>, raw: String) -> EatMode {
    match get_network_name() {
        Some(network) if network.eq_ignore_ascii_case("twitch") => {
            let msg: Message = raw.parse().expect("Failed to parse IRC Message");
            let opt_eat: Option<EatMode> = match &*msg.command {
                //  Chat Messages.
                "PRIVMSG" => {
                    CURRENT.lock().put(msg);
                    Some(EatMode::None)
                }
                "WHISPER" => events::whisper_recv(msg),

                //  Status updates.
                "HOSTTARGET" => events::hosttarget(&word[3]),
                "ROOMSTATE" => events::roomstate(msg),
                "USERNOTICE" => events::usernotice(msg),
                "USERSTATE" => events::userstate(msg),

                //  Moderator Actions.
                "CLEARMSG" => events::clearmsg(msg),
                "CLEARCHAT" => events::clearchat(msg),

                //  Other.
                _ => Some(EatMode::None),
            };

            opt_eat.unwrap_or_else(|| {
                echo(
                    EVENT_ERR,
                    &[&raw],
                    1,
                );
                EatMode::None
            })
        }
        _ => EatMode::None,
    }
}


pub(crate) fn cmd_ht_debug(_arg: &[String]) -> EatMode {
    let new: bool = get_pref_int("PREF_htdebug").unwrap_or(0) == 0;

    if set_pref_int("PREF_htdebug", new.into()).is_ok() {
        if new {
            echo(EVENT_NORMAL, &[
                "Unrecognized UserNotices will now show the full Message.",
            ], 0);
        } else {
            echo(EVENT_NORMAL, &[
                "Unrecognized UserNotices will NOT show the full Message.",
            ], 0);
        }
    } else {
        echo(EVENT_ERR, &["FAILED to set Preference."], 0);
    }

    EatMode::All
}


pub(crate) fn cmd_reward(argslice: &[String]) -> EatMode {
    let arg: Vec<&str> = argslice[1..].iter()
        .take_while(|s| !s.is_empty())
        .map(String::as_str)
        .collect();
    let len = arg.len();

    if len < 1 {
        //  Print the current Reward Names.
        echo(EVENT_NORMAL, &["REWARD EVENTS:"], 0);
        for pref in get_prefs() {
            if !pref.is_empty() && !pref.starts_with("PREF") {
                echo(
                    EVENT_NORMAL,
                    &[format!(
                        "{}: '{}'",
                        pref,
                        get_pref_string(&pref)
                            .unwrap_or_default(),
                    )],
                    0,
                );
            }
        }
    } else if {
        !arg[0].starts_with("PREF")
            && {
            if len < 2 {
                //  Unset a Reward.
                delete_pref(&arg[0].to_lowercase())
            } else {
                //  Set a Reward.
                set_pref_string(
                    &arg[0].to_lowercase(),
                    &arg[1..].join(" ").trim(),
                )
            }
        }.is_ok()
    } {
        echo(EVENT_NORMAL, &["Preference set."], 0);
    } else {
        echo(EVENT_ERR, &["FAILED to set Preference."], 0);
    }

    EatMode::All
}


pub(crate) fn cmd_title(arg: &[String]) -> EatMode {
    send_command(&format!(
        "RECV :Twitch@twitch.tv TOPIC #{} :{}",
        &arg[1].to_ascii_lowercase(),
        &arg[2..].join(" ").trim(),
    ));

    EatMode::All
}


pub(crate) fn cmd_tjoin(arg: &[String]) -> EatMode {
    send_command(&format!(
        "JOIN {}",
        &arg[1..].join(" ").trim(),
    ));

    EatMode::All
}


pub(crate) fn cmd_whisper(arg: &[String]) -> EatMode {
    if arg.len() > 1
        && get_network_name().unwrap_or_default()
        .eq_ignore_ascii_case("twitch")
    {
        let targ: &str = &arg[1];

        //  Two stage assignment to prevent Temporary Value.
        let tmp: String = arg[2..].join(" ");
        let msg: &str = tmp.trim();

        //  Check for trailing Arguments.
        if msg.is_empty() {
            //  None: Switch to Whisper Tab.
            send_command(&format!("QUERY {}", targ));
        } else {
            //  Some: Send through Whisper.
            send_command(&format!("SAY .w {} {}", targ, msg));
        }
    }
    EatMode::All
}


pub(crate) fn cmd_whisper_here(_arg: &[String]) -> EatMode {
    let new: bool = get_pref_int("PREF_whispers_in_current").unwrap_or(0) == 0;

    if set_pref_int("PREF_whispers_in_current", new.into()).is_ok() {
        if new {
            echo(EVENT_NORMAL, &["Twitch Whispers will also show in the current Tab."], 0);
        } else {
            echo(EVENT_NORMAL, &["Twitch Whispers will ONLY be shown in their own Tabs."], 0);
        }
    } else {
        echo(EVENT_ERR, &["FAILED to set Preference."], 0);
    }

    EatMode::All
}
