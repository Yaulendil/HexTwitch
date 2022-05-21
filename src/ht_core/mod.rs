mod events;
mod output;

use std::collections::HashSet;
use chrono::{DateTime, Utc};
use hexchat::{
    ChannelRef,
    EatMode,
    get_channel,
    get_channel_name,
    get_network_name,
    PrintEvent,
    strip_formatting,
};
use parking_lot::Mutex;

use crate::{irc::{Message, Signature}, NETWORK, prefs::*};
use output::{
    alert_basic,
    alert_error,
    BADGES_UNKNOWN,
    PREDICTIONS,
    print_topic,
    print_with_irc,
    print_without_irc,
    TABCOLORS,
};


enum MsgSrc {
    /// This message has already been processed, and is being emitted again. It
    ///     should be passed through unaffected.
    ReEmit,
    /// This message is from the IRC Server. It should be processed, and has IRC
    ///     data associated with it.
    Server(Message),
    /// This message has no associated IRC data, but it has not been processed
    ///     already.
    Unknown,
}


#[derive(Default)]
struct Sponge {
    signature: Option<Signature>,
    previous: Option<Signature>,
    message: Option<Message>,
}

impl Sponge {
    /// Place a Message into the Sponge. The previous Message in the Sponge, if
    ///     any, is removed. Takes Ownership of the new Message.
    ///
    /// Input: [`Message`]
    fn put(&mut self, new: Message) {
        let sig_new = new.get_signature();

        if self.previous.as_ref() == Some(&sig_new) {
            //  New message is a re-emit of the last message finished. Ignore
            //      it, but do NOT ignore the NEXT reoccurrence of the same
            //      signature.
            self.previous.take();
        } else {
            //  New message does not match the last emitted. Accept it.
            self.signature.replace(sig_new);
            self.message.replace(new);
        }
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
    /// Input: [`&Signature`]
    /// Return: [`MsgSrc`]
    ///
    /// [`&Signature`]: Signature
    fn pop(&mut self, signature: &Signature) -> MsgSrc {
        //  Check the last processed message signature. At the same time, clear
        //      the last processed, because this check is the sole purpose for
        //      its existence.
        match self.previous.take() {
            //  New signature matches previous. Probably a re-emit.
            Some(old) if signature == &old => MsgSrc::ReEmit,

            //  Compare against the currently-held signature.
            _ => match &self.signature {
                Some(sig) if signature == sig => match self.message.take() {
                    //  Signature matches and message is present. Return it.
                    Some(msg) => MsgSrc::Server(msg),

                    //  Signature matches, but message is already gone.
                    None => MsgSrc::Unknown,
                }

                //  Signature does not match.
                Some(_) => MsgSrc::Unknown,
                //  Not holding a signature.
                None => MsgSrc::Unknown,
            }
        }
    }

    fn set_prev(&mut self, signature: Signature) {
        self.previous = Some(signature);
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


fn check_message(channel: &str, author: &str) -> MsgSrc {
    CURRENT.lock().pop(&Signature::new(
        Some(channel),
        strip_formatting(author),
    ))
}


fn mark_processed(sig: Signature) {
    CURRENT.lock().set_prev(sig);
}


fn set_topic(channel: &str, topic: &str) {
    if get_channel(NETWORK, channel).is_some() {
        print_topic(channel);
        cmd!("RECV :Twitch!twitch@twitch.tv TOPIC {} :{}", channel, topic);
    }
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
        TABCOLORS.reset();
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
    fn need_irc(etype: PrintEvent) -> bool {
        match etype {
            PrintEvent::YOUR_ACTION => false,
            PrintEvent::YOUR_MESSAGE => false,
            _ => true,
        }
    }

    if this_is_twitch() {
        let channel: String = get_channel_name();
        let author: &String = &word[0];

        //  Determine what should be done with this message.
        match check_message(&channel, author) {
            //  Message was already processed. Ignore it.
            MsgSrc::ReEmit => EatMode::None,
            //  Message comes from Server. IRC Representation available.
            MsgSrc::Server(msg) => print_with_irc(&channel, etype, word, msg),

            //  No IRC Representation available for Message.
            MsgSrc::Unknown if need_irc(etype) => EatMode::None,
            MsgSrc::Unknown => print_without_irc(&channel, etype, word),
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
    let new: bool = PREF_DEBUG.get() != Some(true);

    if PREF_DEBUG.set(new).is_ok() {
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


pub fn cmd_prediction(_arg_full: &[String]) -> EatMode {
    alert_basic(&format!(
        "Current Prediction: {}",
        PREDICTIONS.get(&get_channel_name()),
    ));

    EatMode::All
}


pub fn cmd_reward(arg_full: &[String]) -> EatMode {
    match arg_trim(&arg_full[1..]) {
        [] => {
            //  Print the current Reward Names.
            alert_basic("REWARD EVENTS:");

            // for pref in get_prefs() {
            //     if !pref.is_empty() && !pref.starts_with(Pref::PREFIX) {
            //         alert_basic(&format!(
            //             "{}: '{}'",
            //             pref,
            //             get_pref_string(&pref).unwrap_or_default(),
            //         ));
            //     }
            // }

            for reward in Reward::get_all() {
                alert_basic(&format!(
                    "{}: '{}'",
                    reward.id(),
                    reward.get().unwrap_or_default(),
                ));
            }
        }
        [uuid, content @ ..] => match uuid.parse::<Reward>() {
            Ok(reward) => match if content.is_empty() {
                //  Unset a Reward.
                reward.unset()
            } else {
                //  Set a Reward.
                reward.set(&content.join(" "))
            } {
                Ok(()) => alert_basic("Reward updated."),
                Err(()) => alert_error("FAILED to update Reward."),
            }
            Err(()) => alert_error("Invalid Reward ID."),
        }
    }

    EatMode::All
}


pub fn cmd_title(arg_full: &[String]) -> EatMode {
    let channel: String = arg_full[1].to_ascii_lowercase();
    let topic: String = arg_trim(&arg_full[2..]).join(" ");

    set_topic(&channel, &topic);

    EatMode::All
}


pub fn cmd_tjoin(arg_full: &[String]) -> EatMode {
    cmd!(
        "JOIN {}",
        arg_trim(&arg_full[1..]).join(" "),
    );

    EatMode::All
}


pub fn cmd_unk_badges(_arg_full: &[String]) -> EatMode {
    let unk: &HashSet<String> = &BADGES_UNKNOWN.get();

    if unk.is_empty() {
        alert_basic("No unknown Badges have been seen.");
    } else {
        alert_basic("The following Badges do not have associated icons:");

        let mut vec: Vec<&String> = unk.iter().collect();
        vec.sort_unstable();

        for s in vec {
            alert_basic(s);
        }
    }

    EatMode::All
}


pub fn cmd_whisper(arg_full: &[String]) -> EatMode {
    let arg: &[String] = arg_trim(arg_full);

    match arg {
        [] | [_] => {}
        _ if !this_is_twitch() => {}
        [_, name] => {
            //  Switch to Whisper Tab.
            cmd!("QUERY {}", name);
        }
        [_, name, words @ ..] => {
            //  Send through Whisper.
            cmd!("SAY .w {} {}", name, words.join(" "));
        }
    }

    EatMode::All
}


pub fn cmd_whisper_here(_arg_full: &[String]) -> EatMode {
    let new: bool = !PREF_WHISPERS.is(true);

    if PREF_WHISPERS.set(new).is_ok() {
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
