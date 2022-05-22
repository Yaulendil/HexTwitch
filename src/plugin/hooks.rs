use chrono::{DateTime, Utc};
use hexchat::{
    add_print_event_listener,
    add_raw_server_event_listener,
    add_window_event_listener,
    ChannelRef,
    Command,
    deregister_command,
    EatMode,
    PrintEvent,
    PrintEventListener,
    Priority,
    RawServerEventListener,
    register_command,
    remove_print_event_listener,
    remove_raw_server_event_listener,
    remove_window_event_listener,
    WindowEvent,
    WindowEventListener,
};


//  This is far too overengineered. At least that means it can later be reused
//      in other projects.
macro_rules! trait_alias {
    {$(
        $(#[$attr:meta])*
        $v:vis trait $alias:ident
        $(<$($T:ident $(: $bound_first:tt $(+ $bounds:tt)*)?),+>)?
        = { $($target:tt)* };
    )*} => {$(
        $(#[$attr])*
        $v trait $alias$(<$($T),+>)?: $($target)* where
        $($($($T: $bound_first $(+ $bounds)*,)?)+)?
        {}

        impl<$($($T,)+)? __TARGET__> $alias$(<$($T),+>)? for __TARGET__ where
        __TARGET__: $($target)*,
        $($($($T: $bound_first $(+ $bounds)*,)?)+)?
        {}
    )*};
}

trait_alias! {
    /// A callback function to be run by a HexChat [`Command`].
    pub trait CbCommand = {
        Fn(&[String]) -> EatMode
        + 'static
    };

    /// A callback function to be run on a [`PrintEvent`].
    pub trait CbPrint = {
        Fn(&[String], DateTime<Utc>) -> EatMode
        + 'static
    };

    /// A function to run on a [`PrintEvent`]. This cannot be passed directly to
    ///     HexChat yet, because HexChat assumes that callbacks should have no
    ///     need for the PrintEvent. It must first be [`wrapped`] in a
    ///     [`CbPrint`] function.
    ///
    /// [`wrapped`]: wrap_print
    pub trait CbPrintPlugin = {
        Fn(PrintEvent, &[String], DateTime<Utc>) -> EatMode
        + 'static
    };

    /// A callback function to be run on a server event.
    pub trait CbServer = {
        Fn(&[String], DateTime<Utc>, String) -> EatMode
        + 'static
    };

    /// A callback function to be run on a [`WindowEvent`].
    pub trait CbWindow = {
        Fn(ChannelRef) -> EatMode
        + 'static
    };
}


pub fn wrap_print(event: PrintEvent, cb: impl CbPrintPlugin) -> impl CbPrint {
    move |words: &[String], dt: DateTime<Utc>| cb(event, words, dt)
}


pub enum Hook {
    Command(Command),
    Print(PrintEventListener),
    Server(RawServerEventListener),
    Window(WindowEventListener),
}

impl Hook {
    const PRIORITY: Priority = Priority::NORMAL;

    pub fn command(name: &str, help: &str, cb: impl CbCommand) -> Self {
        HookCommand::new(name, help, cb).register(Self::PRIORITY)
    }

    pub fn print(event: PrintEvent, cb: impl CbPrint) -> Self {
        HookPrint::new(event, cb).register(Self::PRIORITY)
    }

    pub fn server(event: &str, cb: impl CbServer) -> Self {
        HookServer::new(event, cb).register(Self::PRIORITY)
    }

    pub fn window(event: WindowEvent, cb: impl CbWindow) -> Self {
        HookWindow::new(event, cb).register(Self::PRIORITY)
    }
}

impl Hook {
    pub fn unhook(self) {
        match self {
            Self::Command(handle) => { deregister_command(handle) }
            Self::Print(handle) => { remove_print_event_listener(handle) }
            Self::Server(handle) => { remove_raw_server_event_listener(handle) }
            Self::Window(handle) => { remove_window_event_listener(handle) }
        }
    }
}


trait Hookable: Sized {
    fn register(self, pri: Priority) -> Hook;
}


/// [`Command`] hook.
pub struct HookCommand<'h, F: CbCommand> {
    name: &'h str,
    help: &'h str,
    /// Callback function to be executed by HexChat.
    cb: F,
}

impl<'h, F: CbCommand> HookCommand<'h, F> {
    pub fn new(name: &'h str, help: &'h str, cb: F) -> Self {
        Self { name, help, cb }
    }
}

impl<'h, F: CbCommand> Hookable for HookCommand<'h, F> {
    fn register(self, pri: Priority) -> Hook {
        Hook::Command(register_command(self.name, self.help, pri, self.cb))
    }
}


/// [`PrintEvent`] hook.
pub struct HookPrint<F: CbPrint> {
    /// The event type to listen for.
    event: PrintEvent,
    /// Callback function to be executed by HexChat.
    cb: F,
}

impl<F: CbPrint> HookPrint<F> {
    pub fn new(event: PrintEvent, cb: F) -> Self {
        Self { event, cb }
    }
}

impl<F: CbPrint> Hookable for HookPrint<F> {
    fn register(self, pri: Priority) -> Hook {
        Hook::Print(add_print_event_listener(self.event, pri, self.cb))
    }
}


/// Server Event hook.
pub struct HookServer<'h, F: CbServer> {
    /// The event type to listen for.
    event: &'h str,
    /// Callback function to be executed by HexChat.
    cb: F,
}

impl<'h, F: CbServer> HookServer<'h, F> {
    pub fn new(event: &'h str, cb: F) -> Self {
        Self { event, cb }
    }
}

impl<'h, F: CbServer> Hookable for HookServer<'h, F> {
    fn register(self, pri: Priority) -> Hook {
        Hook::Server(add_raw_server_event_listener(self.event, pri, self.cb))
    }
}


/// [`WindowEvent`] hook.
pub struct HookWindow<F: CbWindow> {
    /// The event type to listen for.
    event: WindowEvent,
    /// Callback function to be executed by HexChat.
    cb: F,
}

impl<F: CbWindow> HookWindow<F> {
    pub fn new(event: WindowEvent, cb: F) -> Self {
        Self { event, cb }
    }
}

impl<F: CbWindow> Hookable for HookWindow<F> {
    fn register(self, pri: Priority) -> Hook {
        Hook::Window(add_window_event_listener(self.event, pri, self.cb))
    }
}
