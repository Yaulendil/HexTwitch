//! Storage of IRC [`Message`]s across the period between reception and output.

use hexchat::EatMode;
use parking_lot::Mutex;
use crate::irc::Message;


safe_static! {
    static lazy CURRENT: Mutex<Storage> = Default::default();
}


/// After calling this function, the next print event will be suppressed.
pub fn eat_next_print_event() {
    CURRENT.lock().set_next(EatMode::All);
}


/// After calling this function, the next print event will not be processed.
pub fn ignore_next_print_event() {
    CURRENT.lock().set_next(EatMode::None);
}


// /// Nullifies the effect of previous calls to ignore functions.
// pub fn process_next_print_event() {
//     CURRENT.lock().next = None;
// }


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
    /// Eat the event.
    Eat(EatMode),
    /// Process the event without IRC, if possible.
    ProcPrint,
    /// Process the event with an IRC [`Message`] for context.
    ProcIrc(Message),
}


#[derive(Default)]
struct Storage {
    irc: Option<Message>,
    next: Option<EatMode>,
}

impl Storage {
    fn get(&mut self) -> Action {
        match self.next.take() {
            Some(eat) => Action::Eat(eat),
            None => match self.irc.take() {
                Some(msg) => Action::ProcIrc(msg),
                None => Action::ProcPrint,
            }
        }
    }

    fn put(&mut self, msg: Message) {
        self.irc = Some(msg);
    }

    fn set_next(&mut self, eat: EatMode) {
        self.next = Some(eat);
    }
}
