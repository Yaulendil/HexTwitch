use crate::prefs::HexPrefGet;

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
}

#[allow(dead_code)]
impl MenuGroup {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into(), subpaths: Vec::new() }
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
}

impl MenuItem for MenuGroup {
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

impl MenuItem for MenuCommand {
    fn path(&self) -> String {
        String::from(self.desc)
    }

    fn tail(&self) -> String {
        format!("{:?}", self.cmd)
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
        let initial = if self.pref.get() == Some(true) {
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
