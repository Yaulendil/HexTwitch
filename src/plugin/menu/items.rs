use crate::{icons::Icon, prefs::HexPrefGet};


pub trait MenuItem: Sized {
    /// Indicates whether this item type will deregister its own name when it is
    ///     dropped. If it does not, it will need to be deregistered by whatever
    ///     called its [`add()`] method.
    ///
    /// This behavior is NOT automatic; it is possible for this constant to lie,
    ///     and any implementor that sets it to `true` should have a [`Drop`]
    ///     impl.
    ///
    /// [`add()`]: Self::add
    const CLEANS_SELF: bool = false;

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
    pos: Option<isize>,
}

#[allow(dead_code)]
impl MenuGroup {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            subpaths: Vec::new(),
            icon: None,
            pos: None,
        }
    }

    pub fn add_item<M: MenuItem>(&mut self, item: M) {
        let opts = item.opts();
        let path = item.path();
        let tail = item.tail();

        let subpath = format!("{}/{}", self.path(), path);

        menu_add!(opts, subpath, tail);

        if !M::CLEANS_SELF {
            self.subpaths.push(subpath);
        }
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

    pub fn with_pos(mut self, position: isize) -> Self {
        self.pos = Some(position);
        self
    }
}

impl MenuItem for MenuGroup {
    const CLEANS_SELF: bool = true;

    fn opts(&self) -> String {
        let mut opts = Vec::with_capacity(2);

        if let Some(icon) = &self.icon {
            if let Some(path) = icon.path_asset() {
                opts.push(format!("-i{}", path.display()));
            }
        }

        if let Some(p) = self.pos {
            opts.push(format!("-p{}", p));
        }

        opts.join(" ")
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
