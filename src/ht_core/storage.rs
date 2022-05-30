//! Storage of IRC [`Message`]s across the period between reception and output.

use parking_lot::Mutex;
use crate::irc::Message;


safe_static! {
    static lazy CURRENT: Mutex<Storage> = Default::default();
}


/// After calling this function, the next print event will not be processed.
pub fn ignore_next_print_event() {
    CURRENT.lock().skip_next();
}


/// Get the IRC [`Message`], if it is set.
pub fn recover_message() -> Action {
    CURRENT.lock().get()
}


/// Store a new IRC [`Message`], to be retrieved during a later print event.
pub fn store_message(msg: Message) {
    CURRENT.lock().put(msg);
}


/// What to do after reading the [`Storage`] state.
pub enum Action {
    // /// Eat the event.
    // Eat(EatMode),
    /// Ignore the event.
    Ignore,
    /// Process the event without IRC, if possible.
    ProcPrint,
    /// Process the event with an IRC [`Message`] for context.
    ProcIrc(Message),
}


#[derive(Default)]
struct Storage {
    irc: Option<Message>,
    skip: bool,
}

impl Storage {
    fn get(&mut self) -> Action {
        if self.swap_skip() {
            Action::Ignore
        } else {
            match self.irc.take() {
                Some(msg) => Action::ProcIrc(msg),
                None => Action::ProcPrint,
            }
        }
    }

    fn put(&mut self, msg: Message) {
        self.irc = Some(msg);
    }

    fn skip_next(&mut self) {
        self.skip = true;
    }

    fn swap_skip(&mut self) -> bool {
        let value = self.skip;
        self.skip = false;
        value
    }
}
