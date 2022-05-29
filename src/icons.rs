use std::{fs::{create_dir_all, File}, io::Write, path::{Path, PathBuf}};
use hexchat::get_config_dir;


const DIR_ICONS: &str = "icons_hextwitch";

macro_rules! img {($path:literal) => {concat!("../img/", $path)}}
macro_rules! icon {
    ($path:literal) => {icon!($path, $path)};
    ($path_load:literal, $path_save:literal) => {&Icon::new(
        $path_save,
        include_bytes!(img!($path_load)),
    )};
}

pub static I_MOD: &Icon = icon!("badges/moderator.png", "mod.png");
pub static I_VIP: &Icon = icon!("badges/vip.png", "vip.png");
pub static I_UNMOD: &Icon = icon!("unmod.png");
pub static I_UNVIP: &Icon = icon!("unvip.png");

pub static I_MODE_ON: &Icon = icon!("ui/lock.svg");
pub static I_MODE_OFF: &Icon = icon!("ui/unlock.svg");
pub static I_PLUS: &Icon = icon!("ui/plus.svg");
pub static I_STOP: &Icon = icon!("ui/stop.svg");

pub static I_CLEAR: &Icon = icon!("ui/clear.svg");
pub static I_INFO: &Icon = icon!("ui/info.svg");
pub static I_RELOAD: &Icon = icon!("ui/reload.svg");
pub static I_TAG: &Icon = icon!("ui/tag-new.svg");

pub static I_PREDICT: &Icon = icon!("ui/show-prediction.svg");
pub static I_REWARDS: &Icon = icon!("ui/show-rewards.svg");
pub static I_UNKNOWN: &Icon = icon!("ui/show-warning.svg");

pub static I_BAN: &Icon = icon!("ui/user-ban.svg");
pub static I_UNBAN: &Icon = icon!("ui/user-unban.svg");
pub static I_TIMEOUT: &Icon = icon!("ui/user-timeout.svg");


pub struct Icon {
    path: &'static str,
    data: &'static [u8],
}

impl Icon {
    const fn new(path: &'static str, data: &'static [u8]) -> Self {
        Self { path, data }
    }

    pub fn path_asset(&self) -> Option<PathBuf> {
        let mut path: PathBuf = get_config_dir();
        path.push(DIR_ICONS);
        path.push(self.path);

        #[cfg(feature = "full-debug")]
        hexchat::print_plain(&path.display().to_string());

        if !path.exists() {
            let parent: &Path = path.parent()?;
            if !parent.exists() {
                create_dir_all(parent).ok()?;
            }

            let mut file = File::create(&path).ok()?;
            file.write_all(self.data).ok()?;
        }

        Some(path)
    }
}
