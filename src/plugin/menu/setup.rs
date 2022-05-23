use crate::prefs::*;
use super::*;


// TODO: Running this command is broken on the HexChat side. Find a workaround.
#[allow(unused)]
macro_rules! getstr {
    ($default:literal, $cmd:literal, $prompt:literal $(,)?) => {concat!(
        "GETSTR",
        " ",
        concat!('"', $default, '"'),
        " ",
        concat!('"', $cmd, '"'),
        " ",
        concat!('"', $prompt, '"'),
    )};
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
        });
        twitch.add_item(MenuCommand {
            cmd: "SAY .vips",
            desc: "List channel VIPs",
        });
    }

    //  Main menu: Section 3: Plugin config.
    {
        twitch.add_separator();
        twitch.add_item(MenuPrefToggle {
            pref: PREF_DEBUG,
            desc: "Enable Debug mode",
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
            desc: "Show Whispers in current tab",
            set: "WHISPERHERE",
            unset: None,
        });
    }

    //  Channel utility submenu.
    {
        twitch_ch_admin.add_item(MenuCommand {
            cmd: "SAY .marker",
            desc: "Set _Marker",
        });
        twitch_ch_admin.add_item(MenuCommand {
            cmd: "SAY .commercial",
            desc: "Run advertisements",
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
            // cmd: getstr!(
            //     "30",
            //     "SAY .slow",
            //     "Enter delay for Slow Mode",
            // ),
            cmd: "SAY .slow",
            desc: "Enable Slo_w mode",
        });
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .slowoff",
            desc: "Disable Slow mode",
        });
        twitch_ch_modes.add_separator();
        twitch_ch_modes.add_item(MenuCommand {
            cmd: "SAY .followers",
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
        desc: "_Ban",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .timeout %s",
        desc: "_Timeout",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .unban %s",
        desc: "_Unban",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .timeout %s 1",
        desc: "_Purge History",
    });
    user.add_separator();
    user.add_item(MenuCommand {
        cmd: "SAY .mod %s",
        desc: "Add Mod",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .unmod %s",
        desc: "Remove Mod",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .vip %s",
        desc: "Add VIP",
    });
    user.add_item(MenuCommand {
        cmd: "SAY .unvip %s",
        desc: "Remove VIP",
    });
    menus.push(user);

    menus
}
