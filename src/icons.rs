use std::{fs::{create_dir_all, File}, io::Write, path::{Path, PathBuf}};
use hexchat::get_config_dir;


const DIR_ICONS: &str = "icons_twitch";

macro_rules! img {($path:literal) => {concat!("../img/", $path)}}
macro_rules! icon {
    ($path:literal) => {icon!($path, $path)};
    ($path_load:literal, $path_save:literal) => {Icon::new(
        $path_save,
        include_bytes!(img!($path_load)),
    )};
}

pub static I_MOD: Icon = icon!("badges/moderator.png", "mod.png");
pub static I_VIP: Icon = icon!("badges/vip.png", "vip.png");
pub static I_UNMOD: Icon = icon!("unmod.png");
pub static I_UNVIP: Icon = icon!("unvip.png");


pub struct Icon {
    path: &'static str,
    data: &'static [u8],
}

impl Icon {
    const fn new(path: &'static str, data: &'static [u8]) -> Self {
        Self { path, data }
    }

    pub fn path(&self) -> &Path {
        Path::new(self.path)
    }

    pub fn path_asset(&self) -> Option<PathBuf> {
        let mut path: PathBuf = get_config_dir();
        path.push(DIR_ICONS);
        path.push(self.path());

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