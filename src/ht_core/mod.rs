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

use crate::NETWORK;
use irc::Message;
use output::{
    alert_basic,
    alert_error,
    BADGES_UNK,
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
    ///     the Signature specified. Valid Message Signatures are in roughly the
    ///     following format:
    ///         `Some("#channel"):Ok("author")`
    ///
    /// Signatures are calculated from as little data as possible, so that they
    ///     can be derived from the minimal representation provided by Hexchat
    ///     Print Hooks. This way, that minimal representation can be associated
    ///     with the much fuller version received through a Server Hook.
    ///
    /// Input: `&str`
    /// Return: `Option<Message>`
    fn pop(&mut self, signature: &str) -> Option<Message> {
        match self.signature.as_ref() {
            Some(sig) if sig == signature => self.value.take(),
            _ => None,
        }
    }
}


safe_static! {
    static lazy CURRENT: Mutex<Sponge> = Default::default();
}


/// Trim a slice of arguments from Hexchat into something workable. The initial
///     slice likely has only a few arguments that are not empty, with the rest
///     being placeholders left over from when the slice was first constructed
///     in C.
#[inline]
fn arg_trim(args: &[String]) -> &[String] {
    match args.iter().position(String::is_empty) {
        Some(i) => &args[..i],
        None => args,
    }
}


fn check_message(channel: &str, author: &str) -> Option<Message> {
    CURRENT.lock().pop(
        &format!("Some({:?}):{:?}", channel, strip_formatting(author))
    )
}


/// Plugin preferences do not support a boolean type. This function wraps an
///     Integer preference to serve the same purpose. A preference with any non-
///     zero value will be interpreted as `true`. Unset preferences will result
///     in `false`.
#[inline]
fn get_pref_bool(name: &str) -> bool {
    get_pref_int(name).unwrap_or(0) != 0
}


/// Plugin preferences do not support a boolean type. This function wraps an
///     Integer preference to serve the same purpose. Stores either `1` or `0`
///     in the preference, cast directly from the boolean input. Will return
///     `Ok(())` on success, or `Err(())` otherwise.
#[inline]
fn set_pref_bool(name: &str, value: bool) -> Result<(), ()> {
    set_pref_int(name, value as _)
}


fn this_is_twitch() -> bool {
    match get_network_name() {
        Some(network) => network == NETWORK,
        None => false,
    }
}


/// Reset the Color of a newly-focused Tab.
pub fn cb_focus(_channel: ChannelRef) -> EatMode {
    if this_is_twitch() {
        TABCOLORS.lock().reset();
    }
    EatMode::None
}


/// Hide a Join Event if it is fake.
pub fn cb_join(_etype: PrintEvent, word: &[String]) -> EatMode {
    if this_is_twitch() && !word[2].contains("tmi.twitch.tv") {
        EatMode::All
    } else {
        EatMode::None
    }
}


pub fn cb_print(etype: PrintEvent, word: &[String]) -> EatMode {
    if this_is_twitch() {
        let channel: String = get_channel_name();

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
    } else {
        EatMode::None
    }
}


/// Handle a Server Message, received by the Hook for "RAW LINE".
pub fn cb_server(_word: &[String], _dt: DateTime<Utc>, raw: String) -> EatMode {
    if this_is_twitch() {
        let msg: Message = raw.parse().expect("Failed to parse IRC Message");
        let opt_eat: Option<EatMode> = match msg.command.as_str() {
            //  Chat Messages.
            "PRIVMSG" => {
                CURRENT.lock().put(msg);
                Some(EatMode::None)
            }
            "WHISPER" => events::whisper_recv(msg),

            //  Status updates.
            "HOSTTARGET" => events::hosttarget(msg),
            "RECONNECT" => events::reconnect(msg),
            "ROOMSTATE" => events::roomstate(msg),
            "USERNOTICE" => events::usernotice(msg),
            "USERSTATE" => events::userstate(msg),

            //  Moderator Actions.
            "CLEARMSG" => events::clearmsg(msg),
            "CLEARCHAT" => events::clearchat(msg),

            //  Suppress Hexchat spamming complaints that Twitch does not
            //      implement WHO and WHOIS Commands.
            //  TODO: If there is a way to prevent Hexchat from sending a WHO to
            //      every channel after connecting, that would be preferable to
            //      doing this.
            "421" if msg.trail == "Unknown command" =>
                match msg.args.get(1).map(String::as_str) {
                    Some("WHO") | Some("WHOIS") => Some(EatMode::Hexchat),
                    _ => Some(EatMode::None),
                }

            //  Other.
            _ => Some(EatMode::None),
        };

        //  Print the Message if the handler fails to return an EatMode.
        opt_eat.unwrap_or_else(|| {
            //  Do not check for HTDEBUG setting here, because a failure in a
            //      handler, for a known type, is a bigger deal than just not
            //      having a handler for an unknown one. This needs to be
            //      noticed and fixed.
            alert_error(&format!("Handler for IRC Command failed: {}", raw));
            EatMode::None
        })
    } else {
        EatMode::None
    }
}


pub fn cmd_ht_debug(_arg_full: &[String]) -> EatMode {
    let new: bool = !get_pref_bool("PREF_htdebug");

    if set_pref_bool("PREF_htdebug", new).is_ok() {
        alert_basic(
            if new {
                "Unrecognized UserNotices will now show the full Message."
            } else {
                "Unrecognized UserNotices will NOT show the full Message."
            }
        );
    } else {
        alert_error("FAILED to set Preference.");
    }

    EatMode::All
}


pub fn cmd_reward(arg_full: &[String]) -> EatMode {
    let arg: &[String] = arg_trim(&arg_full[1..]);
    let len: usize = arg.len();

    if len < 1 {
        //  Print the current Reward Names.
        alert_basic("REWARD EVENTS:");
        for pref in get_prefs() {
            if !pref.is_empty() && !pref.starts_with("PREF") {
                alert_basic(&format!(
                    "{}: '{}'",
                    pref,
                    get_pref_string(&pref)
                        .unwrap_or_default(),
                ));
            }
        }
    } else if !arg[0].starts_with("PREF")
        &&
        {
            if len < 2 {
                //  Unset a Reward.
                delete_pref(&arg[0].to_lowercase())
            } else {
                //  Set a Reward.
                set_pref_string(
                    &arg[0].to_lowercase(),
                    &arg_trim(&arg[1..]).join(" "),
                )
            }
        }.is_ok()
    {
        alert_basic("Preference set.");
    } else {
        alert_error("FAILED to set Preference.");
    }

    EatMode::All
}


pub fn cmd_title(arg_full: &[String]) -> EatMode {
    send_command(&format!(
        "RECV :Twitch!twitch@twitch.tv TOPIC #{} :{}",
        &arg_full[1].to_ascii_lowercase(),
        arg_trim(&arg_full[2..]).join(" "),
    ));

    EatMode::All
}


pub fn cmd_tjoin(arg_full: &[String]) -> EatMode {
    send_command(&format!(
        "JOIN {}",
        arg_trim(&arg_full[1..]).join(" "),
    ));

    EatMode::All
}


pub fn cmd_unk_badges(_arg_full: &[String]) -> EatMode {
    let unk = BADGES_UNK.read();

    if unk.is_empty() {
        alert_basic("No unknown Badges have been seen.");
    } else {
        alert_basic("The following Badges do not have associated icons:");

        let mut vec: Vec<&str> = unk.iter().map(String::as_str).collect();
        vec.sort_unstable();
        vec.into_iter().for_each(alert_basic);
    }

    EatMode::All
}


pub fn cmd_whisper(arg_full: &[String]) -> EatMode {
    let arg: &[String] = arg_trim(arg_full);

    if arg.len() > 1 && this_is_twitch() {
        let msg: String = arg[2..].join(" ");

        //  Check for message.
        if msg.is_empty() {
            //  None: Switch to Whisper Tab.
            send_command(&format!("QUERY {}", arg[1]));
        } else {
            //  Some: Send through Whisper.
            send_command(&format!("SAY .w {} {}", arg[1], msg));
        }
    }
    EatMode::All
}


pub fn cmd_whisper_here(_arg_full: &[String]) -> EatMode {
    let new: bool = !get_pref_bool("PREF_whispers_in_current");

    if set_pref_bool("PREF_whispers_in_current", new).is_ok() {
        alert_basic(
            if new {
                "Twitch Whispers will also show in the current Tab."
            } else {
                "Twitch Whispers will ONLY be shown in their own Tabs."
            }
        );
    } else {
        alert_error("FAILED to set Preference.");
    }

    EatMode::All
}
