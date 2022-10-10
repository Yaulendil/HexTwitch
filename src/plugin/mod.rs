mod hooks;
mod menu;

use hexchat::{Plugin, print_plain, PrintEvent, WindowEvent};
use crate::{api::API, ht_core::*};
use hooks::{CbCommand, CbPrint, CbPrintPlugin, CbServer, CbWindow, Hook};
use menu::*;


pub struct HexTwitch {
    hooks: Vec<Hook>,
    #[allow(dead_code)]
    menus: Vec<MenuGroup>,
    // api: ApiHandler,
}

impl HexTwitch {
    fn hook_command(&mut self, name: &str, help: &str, cb: impl CbCommand) {
        self.register(Hook::command(name, help, cb));
    }

    fn hook_print(&mut self, event: PrintEvent, cb: impl CbPrint) {
        self.register(Hook::print(event, cb));
    }

    fn hook_print_plugin(&mut self, event: PrintEvent, cb: impl CbPrintPlugin) {
        self.hook_print(event, hooks::wrap_print(event, cb))
    }

    fn hook_server(&mut self, event: &str, cb: impl CbServer) {
        self.register(Hook::server(event, cb));
    }

    fn hook_window(&mut self, event: WindowEvent, cb: impl CbWindow) {
        self.register(Hook::window(event, cb));
    }

    fn register(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }
}

impl Plugin for HexTwitch {
    const NAME: &'static str = env!("CARGO_PKG_NAME");
    const DESC: &'static str = env!("CARGO_PKG_DESCRIPTION");
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn new() -> Self {
        crate::prefs::migrate_prefs();
        crate::prefs::init_prefs();

        let mut plugin = Self {
            hooks: Vec::with_capacity(32),
            menus: create_menus(),
            // api: ApiHandler::new(),
        };

        // plugin.hook_command(
        //     "HTSTATUS",
        //     "Read the channel status from the Twitch API.",
        //     cmd_api_read_status,
        // );

        plugin.hook_command(
            "TWITCHAUTH",
            "Authorize the Twitch API.",
            cmd_api_check,
        );
        plugin.hook_command(
            "TWITCHTAGS",
            "Read the stream tags of the current channel.",
            wrap_threaded(wrap_api_cmd(cmd_api_tags)),
        );

        //  Register Plugin Commands, with helptext.
        plugin.hook_command(
            "HTANNOUNCE",
            "Toggle whether Twitch Announcements should be distinctly colored.",
            cmd_pref_announce,
        );
        plugin.hook_command(
            "HTDEBUG",
            "Toggle whether extra debug information should be printed.",
            cmd_pref_debug,
        );
        plugin.hook_command(
            "HTMODES",
            "Automatically assign fake IRC modes to channel moderators.",
            cmd_automodes,
        );
        plugin.hook_command(
            "HTINFO",
            "Print information about the HexTwitch plugin.",
            cmd_htinfo,
        );
        plugin.hook_command(
            "HOSTFOLLOW",
            "Toggle whether Twitch Hosts will be followed through to the target channel.",
            cmd_pref_follow_hosts,
        );
        plugin.hook_command(
            "PREDICTION",
            "View the current Prediction of the current Twitch Channel.",
            cmd_prediction,
        );
        plugin.hook_command(
            "REWARD",
            "Set the Name of a Custom Reward.\n\n\
                Usage: REWARD <UUID> [<NAME>]",
            cmd_reward,
        );
        plugin.hook_command(
            "TITLE",
            "Set the Title of a Twitch Channel. Intended for use by external \
            tools that read the Twitch API.\n\n\
                Usage: TITLE <channel> <text>",
            cmd_title,
        );
        plugin.hook_command(
            "TWITCHJOIN",
            "Join a Channel, but only on the Twitch Network.",
            cmd_tjoin,
        );
        plugin.hook_command(
            "W",
            "Open a Whisper with a Twitch User.\n\n\
                Usage: W <username> [<message>]",
            cmd_whisper,
        );
        plugin.hook_command(
            "WHISPERHERE",
            "Toggle whether Twitch Whispers should be duplicated in the current Tab.",
            cmd_pref_whisper_here,
        );
        plugin.hook_command(
            "UNKNOWNS",
            "Display unknown Badge Keys that have been seen.",
            cmd_unk_badges,
        );

        //  Hook for Server Notices.
        plugin.hook_print(PrintEvent::SERVER_NOTICE, cb_notice);

        //  Hook for User Joins.
        plugin.hook_print(PrintEvent::JOIN, cb_join);

        //  Hooks for User Modes.
        plugin.hook_print(PrintEvent::CHANNEL_OPERATOR, cb_mode);
        plugin.hook_print(PrintEvent::CHANNEL_DEOP, cb_mode);

        //  Hooks for User Messages.
        plugin.hook_print_plugin(PrintEvent::CHANNEL_MESSAGE, cb_print);
        plugin.hook_print_plugin(PrintEvent::CHANNEL_ACTION, cb_print);
        plugin.hook_print_plugin(PrintEvent::CHANNEL_MSG_HILIGHT, cb_print);
        plugin.hook_print_plugin(PrintEvent::CHANNEL_ACTION_HILIGHT, cb_print);
        plugin.hook_print_plugin(PrintEvent::YOUR_MESSAGE, cb_print);
        plugin.hook_print_plugin(PrintEvent::YOUR_ACTION, cb_print);

        //  Hook RAW LINE Server Messages into the general Handler Callback.
        plugin.hook_server("RAW LINE", cb_server);

        //  Hook Tab Focus events.
        plugin.hook_window(WindowEvent::FOCUS_TAB, cb_focus);

        //  Report loadedness.
        print_plain(&format!("{} {} loaded.", Self::NAME, Self::VERSION));
        plugin
    }
}

impl Drop for HexTwitch {
    fn drop(&mut self) {
        API.write().stop();
        self.hooks.drain(..).for_each(Hook::unhook);
    }
}
