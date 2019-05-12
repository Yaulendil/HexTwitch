#[macro_use]
extern crate hexchat_plugin;

use std::sync::Mutex;
use hexchat_plugin::{Plugin, PluginHandle, CommandHookHandle};

#[derive(Default)]
struct HexTwitch {
    cmd: Mutex<Option<CommandHookHandle>>
}

impl Plugin for HexTwitch {
    fn init(&self,
            ph: &mut PluginHandle,
            _arg: Option<&str>
    ) -> bool {
        ph.register(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_DESCRIPTION"),
            env!("CARGO_PKG_VERSION")
        );
        *self.cmd.lock().unwrap() = Some(ph.hook_command(
            "asdf",
            |
                ph,
                _arg,
                _arg_eol
            | {
                ph.print("qwert");
                hexchat_plugin::EAT_ALL
            },
            hexchat_plugin::PRI_NORM,
            Some("prints 'qwert'")
        ));
        true
    }
}

hexchat_plugin!(HexTwitch);

//fn main() { } // satisfy the compiler, we can't actually run the code
