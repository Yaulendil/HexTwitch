macro_rules! menu {
    ($kw:literal, $opts:expr, $path:expr, $tail:expr $(,)?) => {format!(
        "MENU {opts} {kw} {path:?} {tail}",
        kw = $kw,
        opts = $opts,
        path = $path,
        tail = $tail,
    )};
    ($kw:literal, $path:expr, $tail:expr $(,)?) => {
        menu!($kw, "", $path, $tail)
    };
    ($kw:literal, $path:expr $(,)?) => {
        menu!($kw, "", $path, "")
    };
    ($kw:literal, struct $item:expr) => {menu!(
        $kw,
        $item.opts(),
        $item.path(),
        $item.tail(),
    )};
}
macro_rules! menu_add {($($t:tt)*) => {menu_run!("ADD", $($t)*)}}
macro_rules! menu_del {($($t:tt)*) => {menu_run!("DEL", $($t)*)}}
macro_rules! menu_run {($($t:tt)*) => {::hexchat::send_command(&menu!($($t)*))}}

mod setup;
mod items;

pub use items::*;
pub use setup::create_menus;
