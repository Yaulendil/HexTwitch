use crate::{icons::Icon, prefs::HexPrefGet};


pub trait MenuItem: Sized {
    fn add(&self) { menu_add!(struct self); }
    fn del(&self) { menu_del!(struct self); }

    fn opts(&self) -> String { String::new() }
    fn path(&self) -> String;
    fn tail(&self) -> String { String::new() }
}


pub struct MenuGroup {
    path: String,
    subpaths: Vec<String>,
    icon: Option<&'static Icon>,
}

#[allow(dead_code)]
impl MenuGroup {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            subpaths: Vec::new(),
            icon: None,
        }
    }

    pub fn add_item(&mut self, item: impl MenuItem) {
        let opts = item.opts();
        let path = item.path();
        let tail = item.tail();

        let subpath = format!("{}/{}", self.path(), path);

        menu_add!(opts, subpath, tail);
        self.subpaths.push(subpath);
    }

    pub fn add_separator(&mut self) {
        let subpath = format!("{}/-", self.path);

        menu_add!(subpath);
        self.subpaths.push(subpath);
    }

    pub fn sub_menu(&mut self, sub: &str) -> Self {
        Self::new(format!("{}/{}", self.path, sub))
    }

    pub fn with_icon(mut self, icon: &'static Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl MenuItem for MenuGroup {
    fn opts(&self) -> String {
        match &self.icon {
            Some(icon) => match icon.path_asset() {
                Some(path) => format!("-i{}", path.display()),
                None => String::new(),
            }
            None => String::new(),
        }
    }

    fn path(&self) -> String { self.path.clone() }
}

impl Drop for MenuGroup {
    fn drop(&mut self) {
        for sub in &self.subpaths {
            menu_del!(sub);
        }

        menu_del!(self.path);
    }
}


pub struct MenuCommand {
    pub cmd: &'static str,
    pub desc: &'static str,
}

impl MenuCommand {
    pub fn with_icon(self, icon: &'static Icon) -> MenuCommandIcon {
        MenuCommandIcon { cmd: self, icon }
    }
}

impl MenuItem for MenuCommand {
    fn path(&self) -> String {
        String::from(self.desc)
    }

    fn tail(&self) -> String {
        format!("\"{}\"", self.cmd.replace("\"", "\"\""))
    }
}


pub struct MenuCommandIcon {
    cmd: MenuCommand,
    icon: &'static Icon,
}

impl MenuItem for MenuCommandIcon {
    fn opts(&self) -> String {
        match self.icon.path_asset() {
            Some(path) => format!("-i{}", path.display()),
            None => String::new(),
        }
    }

    fn path(&self) -> String {
        self.cmd.path()
    }

    fn tail(&self) -> String {
        self.cmd.tail()
    }
}


pub struct MenuPrefToggle<P: HexPrefGet<Output=bool>> {
    pub pref: P,
    pub desc: &'static str,
    pub set: &'static str,
    pub unset: Option<&'static str>,
}

impl<P: HexPrefGet<Output=bool>> MenuItem for MenuPrefToggle<P> {
    fn opts(&self) -> String {
        let initial = if self.pref.is(&true) {
            "-t1"
        } else {
            "-t0"
        };

        format!("{}", initial)
    }

    fn path(&self) -> String {
        self.desc.to_owned()
    }

    fn tail(&self) -> String {
        format!(
            "{set:?} {unset:?}",
            set = self.set,
            unset = self.unset.unwrap_or(self.set),
        )
    }
}
