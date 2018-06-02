
use ini::Ini;
use rayon::prelude::*;
use gtk_icon_cache::GtkIconCache;
use lru_cache::LruCache;

use std::path::{Path, PathBuf};
use std::convert::From;
use std::env;
use std::sync::Mutex;
use std::sync::Arc;

static BASIC_EXTS: &'static [&'static str] = &["png", "svg"];
static EXTRA_EXTS: &'static [&'static str] = &["png", "svg", "xpm"];

lazy_static!{
    static ref USER_ICON_DIR: Vec<PathBuf> = get_user_icon_dir();
    static ref ICON_THEME_CACHE: Mutex<LruCache<String, Arc<IconTheme>>> = Mutex::new(LruCache::new(8));
}

fn get_user_icon_dir() -> Vec<PathBuf> {

    if let Ok(dirs) = env::var("XDG_DATA_DIRS") {
        return dirs.split(':')
                   .map(|x| Into::<PathBuf>::into(x).join("icons"))
                   .filter(|x| x.is_dir())
                   .collect()
    }

    if let Ok(dir) = env::var("XDG_DATA_HOME") {
        let dir: PathBuf = format!("{}/icons", dir).into();
        if dir.is_dir() {
            return vec![dir];
        }
    }

    if let Ok(dir) = env::var("HOME") {
        let dir: PathBuf = format!("{}/.local/share/icons", dir).into();
        if dir.is_dir() {
            return vec![dir];
        }
    }

    vec![]
}

#[derive(Debug, Clone)]
pub struct IconName {
    inner_name: String,
}

impl<T> From<T> for IconName
  where T: AsRef<str> {
    fn from(from: T) -> Self {
        Self { inner_name: from.as_ref().to_string() }
    }
}

impl IconName {
    fn name(&self) -> &str {
        &self.inner_name
    }

    fn fallback(&mut self) -> Option<&IconName> {
        let last_dot = self.inner_name.rfind('.')?;
        let last_dash = self.inner_name[..last_dot].rfind('-')?;

        let _ = self.inner_name.drain(last_dash..last_dot).count();

        Some(self)
    }
}

#[derive(Debug, Clone)]
pub struct IconTheme {
    name: String,
    inherits: Vec<String>,
    extra_dirs: Vec<PathBuf>,
    base_dirs: Vec<PathBuf>,
    sub_dirs: Vec<IconDirectory>,
    gtk_cache: Option<GtkIconCache>,
}

#[derive(Debug, Clone)]
pub struct IconDirectory {
    name: String,
    type_: DirectoryType,
    size: i32,
    scale: i32,
}

#[derive(Debug, Clone)]
enum DirectoryType {
    Fixed,
    Scalable(i32, i32),
    Threshold(i32),
}

impl Default for IconDirectory {
    fn default() -> Self {
        Self {
            name: String::new(),
            type_: DirectoryType::Threshold(2),
            size: 0,
            scale: 1,
        }
    }
}

impl Default for IconTheme {
    fn default() -> Self {

        let extra_dirs = if cfg!(test) { vec![] } else { vec!["/usr/share/pixmaps".into()] };

        Self {
            name: String::new(),
            inherits: vec![],
            extra_dirs,
            base_dirs: vec![],
            sub_dirs: vec![],
            gtk_cache: None,
        }
    }
}

impl IconDirectory {
    fn with_settings<T: AsRef<str>>(settings: &Ini, name: T) -> Self {

        let mut r = Self {
            name: name.as_ref().to_string(),
            ..Default::default()
        };

        let properties = match settings.section(Some(name.as_ref())) {
            Some(props) => props,
            None => return r,
        };

        if let Some(Ok(size)) = properties.get("Size").map(|x| x.parse()) {
            r.size = size;
        }

        if let Some(Ok(scale)) = properties.get("Scale").map(|x| x.parse()) {
            r.scale = scale;
        }

        match properties.get("Type").map(|x| x.as_str()) {
            Some("Fixed") => {
                r.type_ = DirectoryType::Fixed;
            },
            Some("Scalable") => {
                let min = properties.get("MinSize").and_then(|x| x.parse().ok());
                let max = properties.get("MaxSize").and_then(|x| x.parse().ok());

                r.type_ = DirectoryType::Scalable(min.unwrap_or(r.size), max.unwrap_or(r.size))
            },
            Some("Threshold") => {
                let threshold = properties.get("Threshold").and_then(|x| x.parse().ok());

                r.type_ = DirectoryType::Threshold(threshold.unwrap_or(2));
            },
            unknown @ _ => {
                error!("Directory Type is invalid: {:?}", unknown);
            },
        }

        r
    }

    /// DirectoryMatchesSize
    pub fn matches_size(&self, size: i32, scale: i32) -> bool {
        if scale != self.scale {
            return false;
        }

        return match self.type_ {
            DirectoryType::Fixed => self.size == size,
            DirectoryType::Scalable(min, max) => min <= size && max >= size,
            DirectoryType::Threshold(threshold) => (self.size - size).abs() <= threshold,
        };
    }

    /// DirectorySizeDistance
    pub fn size_distance(&self, size: i32, scale: i32) -> i32 {
        match self.type_ {
            DirectoryType::Fixed => {
                (self.size * self.scale - size * scale).abs()
            },
            DirectoryType::Scalable(min, max) => {
                if size * scale < min * scale {
                    min * scale - size * scale
                } else {
                    size * scale - max * scale
                }
            },
            DirectoryType::Threshold(_threshold) => {
                // FIXME
                (self.size * self.scale - size * scale).abs()
            },
        }
    }
}

fn search_gtk_cache_in_dir(dir: &Path) -> Option<GtkIconCache> {

    let f = dir.join("icon-theme").with_extension("cache");
    if !f.is_file() { return None; }

    GtkIconCache::with_file_path(f).ok()
}

impl IconTheme {
    pub fn from_dir<T: AsRef<Path>>(path: T) -> Result<IconTheme, ()> {
        let f = Ini::load_from_file(path.as_ref().join("index").with_extension("theme")).map_err(|_| ())?;

        let mut r = Self {
            base_dirs: vec![path.as_ref().into()],
            gtk_cache: search_gtk_cache_in_dir(path.as_ref()),
            ..Default::default()
        };
        let mut directories = vec![];

        if let Some(properties) = f.section(Some("Icon Theme")) {
            r.name = properties.get("Name").unwrap_or(&String::new()).to_string();

            if let Some(list) = properties.get("Inherits").map(|x| x.split(',')) {
                let inherits: Vec<String> = list.map(|x| x.to_string()).collect();
                r.inherits = inherits;
            }

            if let Some(list) = properties.get("Directories").map(|x| x.split(',')) {
                directories = list.map(|x| x.to_string()).collect();
            }
        };

        r.sub_dirs = directories.iter().map(
            |x| IconDirectory::with_settings(&f, x)
        ).collect();

        Ok(r)
    }

    pub fn from_name<T: AsRef<str>>(name: T) -> Result<Arc<IconTheme>, ()> {

        let name = name.as_ref();
        let mut cache = ICON_THEME_CACHE.lock().unwrap();

        if !cache.contains_key(name) {
            let theme = Self::from_name_interal(name)?;
            let _ = cache.insert(name.to_string(), Arc::new(theme));
        }

        Ok(cache.get_mut(name).unwrap().clone())
    }

    fn from_name_interal<T: AsRef<str>>(name: T) -> Result<IconTheme, ()> {

        let system_dir: PathBuf = if cfg!(test) {
            format!("tests/icons/{}", name.as_ref()).into()
        } else {
            format!("/usr/share/icons/{}", name.as_ref()).into()
        };

        let user_dirs: Vec<PathBuf> =
            if cfg!(test) {
                  get_user_icon_dir()
                    .iter()
                    .map(|x| format!("{}/{}", x.display(), name.as_ref()).into())
                    .filter(|x: &PathBuf| x.is_dir())
                    .collect()
            } else {
                  USER_ICON_DIR
                    .iter()
                    .map(|x| format!("{}/{}", x.display(), name.as_ref()).into())
                    .filter(|x: &PathBuf| x.is_dir())
                    .filter(|x| x.is_dir())
                    .collect()
            };

        // construct new theme object
        let mut theme = if user_dirs.is_empty() {
            Self::from_dir(&system_dir)?
        } else {
            if let Some(dir) = user_dirs
                                .iter()
                                .filter(|x| x.join("index").with_extension("theme").is_file())
                                .next() {
                Self::from_dir(dir)?
            } else {
                return Err(());
            }
        };

        // append user-side dirs
        for dir in user_dirs.iter().filter(|x| x.is_dir()) {
            theme.append_base_dir(dir);
        }

        // append system-side dir
        theme.append_base_dir(&system_dir);

        Ok(theme)
    }

    fn append_base_dir<T: AsRef<Path>>(&mut self, path: T) {

        let p = path.as_ref().into();

        if !self.base_dirs.contains(&p) {
            self.base_dirs.push(p);
        }
    }

    #[cfg(test)]
    pub fn append_extra_lookup_dir<T: AsRef<Path>>(&mut self, path: T) {

        self.extra_dirs.push(path.as_ref().into());
    }

    pub fn parents(&self) -> &Vec<String> {
        &self.inherits
    }

    pub fn lookup_icon(&self, name: &IconName, size: i32, scale: i32) -> Option<PathBuf> {

        let ref name = name.name();
        let ref sub_dirs: Vec<&IconDirectory> = match self.gtk_cache.as_ref().and_then(|x| x.lookup(name)) {
            Some(dirs) => {
                self.sub_dirs.par_iter()
                    .filter(|x| dirs.contains(&&x.name))
                    .collect()
            }
            _ => self.sub_dirs.iter().collect(),
        };

        let r = sub_dirs.par_iter()
            .filter(|sub| sub.matches_size(size, scale))
            .flat_map(|sub| self.base_dirs.par_iter()
                            .map_with(sub, |sub, base| format!("{}/{}", base.display(), sub.name)))
            .map(|p| p.into())
            .filter(|p: &PathBuf| p.is_dir())
            .flat_map(|x| BASIC_EXTS.par_iter()
                            .map_with(x, |x, ext| format!("{}/{}.{}", x.display(), name, ext).into()))
            .find_first(|x: &PathBuf| x.is_file());

        if r.is_some() { return r; }

        // test closest file
        let mut minimal_distance = i32::max_value();
        let mut closest_file: Option<PathBuf> = None;

        'dir: for subdir in sub_dirs.iter() {
            let distance = subdir.size_distance(size, scale);
            if distance >= minimal_distance { continue; }

            'location: for basedir in &self.base_dirs {
                'ext: for ext in BASIC_EXTS {
                    let p: PathBuf = format!("{}/{}/{}.{}", basedir.display(), subdir.name, name, ext).into();

                    if p.is_file() {
                        closest_file = Some(p);
                        minimal_distance =  distance;

                        continue 'dir;
                    }
                }
            }
        }

        if closest_file.is_some() { return closest_file; }

        // test in extra dirs
        for extra_dir in self.extra_dirs.iter().filter(|x| x.is_dir()) {
            for ext in EXTRA_EXTS {
                let p: PathBuf = format!("{}/{}.{}", extra_dir.display(), name, ext).into();

                if p.is_file() {
                    return Some(p);
                }
            }
        }

        // not found
        None
    }

    pub fn lookup_fallback_icon(&self, name: &IconName, size: i32, scale: i32) -> Option<PathBuf> {

        let mut fallback = name.clone();
        while let Some(fallback) = fallback.fallback() {
            if let Some(icon) = self.lookup_icon(fallback, size, scale) {
                return Some(icon);
            }
        }

        // fallback without any size/scale match
        let mut fallback = name.clone();
        while let Some(fallback) = fallback.fallback() {
            for basedir in &self.base_dirs {
                for subdir in &self.sub_dirs {
                    for ext in BASIC_EXTS {
                        let p: PathBuf = format!("{}/{}/{}.{}", basedir.display(), subdir.name, fallback.name(), ext).into();

                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use icon_theme::*;
    use icon_lookup::*;

    use std::env;

    #[test]
    fn test_fetch_user_dir() {
        env::remove_var("HOME");
        env::remove_var("XDG_DATA_HOME");
        env::remove_var("XDG_DATA_DIRS");

        assert!(get_user_icon_dir().is_empty());

        env::set_var("HOME", "tests/fake_home");
        let dirs: Vec<PathBuf> = vec![ "tests/fake_home/.local/share/icons".into() ];
        assert_eq!(get_user_icon_dir(), dirs);

        env::set_var("XDG_DATA_HOME", "tests/fake_home/.local/share");
        assert_eq!(get_user_icon_dir(), dirs);

        env::remove_var("HOME");
        assert_eq!(get_user_icon_dir(), dirs);

        env::set_var("XDG_DATA_DIRS", "tests:tests/fake_home/.local/share");
        let dirs: Vec<PathBuf> = vec![ "tests/icons".into(),
                                       "tests/fake_home/.local/share/icons".into() ];
        assert_eq!(get_user_icon_dir(), dirs);
    }

    #[test]
    fn test_icon_name_fallback() {
        env::set_var("XDG_DATA_DIRS", "tests");

        let mut icon_name = IconName::from("some-icon-name.svg");

        println!("{:?}", icon_name);
        while let Some(icon_name) = icon_name.fallback() {
            println!("{:?}", icon_name);
        }
    }

    #[test]
    fn test_app_icon_lookup() {
        env::set_var("XDG_DATA_DIRS", "tests");

        test_lookup!("themed", "deepin-deb-installer", 32, 1
                    => "tests/icons/themed/apps/32/deepin-deb-installer.svg");
    }

    #[test]
    fn test_in_another_base_dir() {
        env::set_var("XDG_DATA_DIRS", "tests:tests/fake_home/.local/share");

        // clear cache
        ICON_THEME_CACHE.lock().unwrap().clear();

        test_lookup!("themed", "just-in-another-base", 16, 1
                    => "tests/fake_home/.local/share/icons/themed/apps/16/just-in-another-base.png");

        // clear cache
        ICON_THEME_CACHE.lock().unwrap().clear();
    }

    #[test]
    fn test_icon_theme_lru_cache() {
        env::set_var("XDG_DATA_DIRS", "tests");

        let mut cache = ICON_THEME_CACHE.lock().unwrap();
        let capacity = cache.capacity();
        cache.clear();
        cache.set_capacity(1);
        drop(cache);

        assert_eq!(0, ICON_THEME_CACHE.lock().unwrap().len());

        test_lookup!("themed", "test", 48, 1
                    => "tests/icons/themed/apps/48/test.png");
        // cache should have 1 new item.
        assert_eq!(1, ICON_THEME_CACHE.lock().unwrap().len());

        let _ = lookup!("hicolor", "test", 48, 1);
        assert_eq!(1, ICON_THEME_CACHE.lock().unwrap().len());
        assert!(ICON_THEME_CACHE.lock().unwrap().contains_key("hicolor"));
        assert!(!ICON_THEME_CACHE.lock().unwrap().contains_key("themed"));

        // clear cache
        let mut cache = ICON_THEME_CACHE.lock().unwrap();
        cache.clear();
        cache.set_capacity(capacity);
    }

    #[test]
    fn test_lookup_threshold() {
        env::set_var("XDG_DATA_DIRS", "tests");

        let theme = IconTheme::from_name("hicolor").unwrap();

        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 46, 1),
                    Some("tests/icons/hicolor/apps/48/TestAppIcon.png".into()));
        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 50, 1),
                    Some("tests/icons/hicolor/apps/48/TestAppIcon.png".into()));
        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 51, 1),
                    Some("tests/icons/hicolor/apps/scalable/TestAppIcon.svg".into()));
    }

    #[test]
    fn test_name_with_dot() {
        env::set_var("XDG_DATA_DIRS", "tests");

        let theme = IconTheme::from_name("themed").unwrap();

        assert_eq!(theme.lookup_icon(&"name.with.dot".into(), 16, 1),
                    Some("tests/icons/themed/apps/16/name.with.dot.png".into()));
    }

    #[test]
    fn test_closest_file() {
        env::set_var("XDG_DATA_DIRS", "tests");

        let theme = IconTheme::from_name("themed").unwrap();

        assert_eq!(theme.lookup_icon(&"name.with.dot".into(), 48, 1),
                    Some("tests/icons/themed/apps/16/name.with.dot.png".into()));
    }

    #[test]
    fn test_extra_lookup_dir() {
        let mut theme = IconTheme::from_dir("tests/icons/hicolor").unwrap();

        // in default, can't find any match
        assert_eq!(theme.lookup_icon(&"ExtraIcon".into(), 48, 1), None);

        // add extra search dir, we can found it.
        theme.append_extra_lookup_dir("tests/extra-icons");
        assert_eq!(theme.lookup_icon(&"ExtraIcon".into(), 48, 1),
                    Some("tests/extra-icons/ExtraIcon.svg".into()));

        assert_eq!(theme.lookup_icon(&"extraxpm-with-fallback".into(), 48, 1),
                    Some("tests/extra-icons/extraxpm-with-fallback.xpm".into()));

        // fallback
        // TODO:
        // assert_eq!(theme.lookup_icon(&"extraxpm".into(), 48, 1),
                    // Some("tests/extra-icons/extraxpm-with-fallback.xpm".into()));
    }
}
