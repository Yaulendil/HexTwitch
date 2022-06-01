macro_rules! ttv {($command:literal) => {concat!("SAY .", $command)}}

/// Execute a command as if typed into HexChat.
macro_rules! cmd {
    // ($($t:tt)*) => { ::hexchat::send_command(&format!($($t)*)) };
    ($text:literal) => { ::hexchat::send_command($text) };
    ($f:literal, $($t:tt)*) => { ::hexchat::send_command(&format!($f, $($t)*)) };
}

/// Execute a command as if typed into Twitch.
#[allow(unused_macros)]
macro_rules! cmd_ttv {
    ($text:literal) => { ::hexchat::send_command(ttv!($text)) };
    ($f:literal, $($t:tt)*) => { ::hexchat::send_command(&format!($f, $($t)*)) };
}

#[allow(unused_macros)]
macro_rules! cmd_at {
    ($channel:expr, $($t:tt)+) => {
        ::hexchat::send_command(&format!(
            "DOAT {}/{} {}"
            $crate::NETWORK,
            $channel,
            format_args!($($t)*),
        ))
    };
}

/// Print text to the Twitch network tab in HexChat.
#[allow(unused_macros)]
macro_rules! twitch_print {
    ($($t:tt)*) => {
        ::hexchat::send_command(&format!(
            "DOAT {} ECHO {}",
            $crate::NETWORK,
            format_args!($($t)*),
        ))
    };
}
