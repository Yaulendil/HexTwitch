use crate::{icons::*, prefs::*};
use super::*;


macro_rules! quote_args {
    ($cmd:literal$(, $arg:literal)*$(,)?) => {concat!(
        $cmd,
        $(' ', '"', $arg, '"',)*
    )};
}
#[allow(unused)]
macro_rules! getbool {
    ($cmd:literal, $title:literal, $text:literal $(,)?) => {
        quote_args!("GETBOOL", $cmd, $title, $text)
    };
}
#[allow(unused)]
macro_rules! getint {
    ($default:literal, $cmd:literal, $prompt:literal $(,)?) => {
        quote_args!("GETINT", $default, $cmd, $prompt)
    };
}
#[allow(unused)]
macro_rules! getstr {
    ($default:literal, $cmd:literal, $prompt:literal $(,)?) => {
        quote_args!("GETSTR", $default, $cmd, $prompt)
    };
}


pub fn create_menus() -> Vec<MenuGroup> {
    let mut menus = Vec::with_capacity(5);

    let mut twitch = MenuGroup::new("_Twitch");
    let mut twitch_ch_admin = twitch.sub_menu("Channel _Admin");
    let mut twitch_ch_modes = twitch.sub_menu("Channel _Modes");

    //  Main menu: Section 1: Output of static data.
    {
        twitch.add();
        twitch.add_item(MenuCommand {
            cmd: "PREDICTION",
            desc: "Show channel _Prediction",
        });
        twitch.add_item(MenuCommand {
            cmd: "REWARD",
            desc: "Show configured _Rewards",
        });
        twitch.add_item(MenuCommand {
            cmd: "UNKNOWNS",
            desc: "Show unknown _Badge tags",
        });
    }

    //  Main menu: Section 2: Channel utils.
    {
        twitch.add_separator();
        twitch_ch_admin.add();
        twitch_ch_modes.add();
        twitch.add_item(MenuCommand {
            cmd: "SAY .mods",
            desc: "List channel Moderators",
        }.with_icon(&I_MOD));
        twitch.add_item(MenuCommand {
            cmd: "SAY .vips",
            desc: "List channel VIPs",
        }.with_icon(&I_VIP));
    }

    //  Main menu: Section 3: Plugin config.
    {
        twitch.add_separator();
        twitch.add_item(MenuPrefToggle {
            pref: PREF_DEBUG,
            desc: "Enable debug mode",
            set: "HTDEBUG",
            unset: None,
        });
        twitch.add_item(MenuPrefToggle {
            pref: PREF_FOLLOW_HOSTS,
            desc: "Follow hosts",
            set: "HOSTFOLLOW",
            unset: None,
        });
        twitch.add_item(MenuPrefToggle {
            pref: PREF_WHISPERS,
            desc: "Show whispers in current tab",
            set: "WHISPERHERE",
            unset: None,
        });
    }

    //  Main menu: Section 4: Plugin misc.
    {
        twitch.add_separator();
        twitch.add_item(MenuCommand {
            cmd: concat!("RELOAD ", env!("CARGO_PKG_NAME")),
            desc: "Reload plugin",
        });
        twitch.add_item(MenuCommand {
            cmd: "HTINFO",
            desc: "About HexTwitch",
        });
    }

    //  Channel management submenu.
    {
        twitch_ch_admin.add_item(MenuCommand {
            cmd: getstr!(
                " ",
                "SAY .marker",
                "Enter comment for Marker (optional)",
            ),
            desc: "Set a _marker",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: getstr!(
                30,
                "SAY .commercial",
                "Enter duration for ad break (in seconds)",
            ),
            desc: "Run advertisements",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: getstr!(
                "BobRoss",
                "SAY .host",
                "Enter channel to host",
            ),
            desc: "_Host a channel",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: getstr!(
                "BobRoss",
                "SAY .raid",
                "Enter channel to raid",
            ),
            desc: "_Raid a channel",
        });
        twitch_ch_admin.add_separator();
        twitch_ch_admin.add_item(MenuCommand {
            cmd: "SAY .clear",
            desc: "Clear channel history",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: "SAY .unhost",
            desc: "Cancel channel _host",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: "SAY .unraid",
            desc: "Cancel channel _raid",
        });
    }

    //  Channel mode submenu.
    {
        twitch_ch_modes.add_item(MenuCommand {
            cmd: getstr!(
                30,
                "SAY .slow",
                "Enter delay for Slow Mode (in seconds)",
            ),
            desc: "Enable Slo_w mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .slowoff",
            desc: "Disable Slow mode",
        });
        twitch_ch_modes.add_separator();
        twitch_ch_modes.add_item(MenuCommand {
            cmd: getstr!(
                "0m",
                "SAY .followers",
                "Enter minimum follow time",
            ),
            desc: "Enable _Followers mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .followersoff",
            desc: "Disable Followers mode",
        });
        twitch_ch_modes.add_separator();
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .subscribers",
            desc: "Enable _Subscribers mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .subscribersoff",
            desc: "Disable Subscribers mode",
        });
        twitch_ch_modes.add_separator();
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .uniquechat",
            desc: "Enable Uni_que mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .uniquechatoff",
            desc: "Disable Unique mode",
        });
        twitch_ch_modes.add_separator();
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .emoteonly",
            desc: "Enable _Emote mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .emoteonlyoff",
            desc: "Disable Emote mode",
        });
    }

    menus.push(twitch_ch_admin);
    menus.push(twitch_ch_modes);
    menus.push(twitch);

    let mut tab = MenuGroup::new("$TAB/_Twitch Channel");
    tab.add();
    tab.add_item(MenuCommand {
        cmd: "SAY .host %s",
        desc: "_Host",
    });
    tab.add_item(MenuCommand {
        cmd: "SAY .raid %s",
        desc: "_Raid",
    });
    menus.push(tab);

    let mut user = MenuGroup::new("$NICK/_Twitch Actions");
    user.add();
    user.add_item(MenuCommand {
        cmd: "JOIN #%s",
        desc: "Join Chat",
    });
    // user.add_item(MenuCommand {
    //     cmd: "EXEC ",
    //     desc: "View Channel",
    // });
    user.add_separator();
    user.add_item(MenuCommand {
        cmd: "SAY .ban %s",
        desc: "_Ban user",
    });
    user.add_item(MenuCommand {
        cmd: getstr!(
            600,
            "SAY .timeout %s",
            "Enter duration for timeout (in seconds)",
        ),
        desc: "_Timeout user",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .unban %s",
        desc: "_Unban user",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .timeout %s 1",
        desc: "_Purge messages",
    });
    user.add_separator();
    user.add_item(MenuCommand {
        cmd: "SAY .mod %s",
        desc: "Add Moderator",
    }.with_icon(&I_MOD));
    user.add_item(MenuCommand {
        cmd: "SAY .unmod %s",
        desc: "Remove Moderator",
    }.with_icon(&I_UNMOD));
    user.add_item(MenuCommand {
        cmd: "SAY .vip %s",
        desc: "Add VIP",
    }.with_icon(&I_VIP));
    user.add_item(MenuCommand {
        cmd: "SAY .unvip %s",
        desc: "Remove VIP",
    }.with_icon(&I_UNVIP));
    menus.push(user);

    menus
}
