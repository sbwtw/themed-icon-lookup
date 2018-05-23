
use ini::Ini;

use std::path::{Path, PathBuf};
use std::convert::From;
use std::env;

static EXTS: &'static [&'static str] = &["png", "svg"];

lazy_static!{
    static ref USER_ICON_DIR: Vec<PathBuf> = get_user_icon_dir();
}

fn get_user_icon_dir() -> Vec<PathBuf> {

    if let Ok(dirs) = env::var("XDG_DATA_DIRS") {
        return dirs.split(':').map(|x| Into::<PathBuf>::into(x).join("icons")).filter(|x| x.is_dir()).collect()
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

#[derive(Debug, Default)]
pub struct IconTheme {
    name: String,
    inherits: Vec<String>,
    extra_dirs: Vec<PathBuf>,
    base_dirs: Vec<PathBuf>,
    sub_dirs: Vec<IconDirectory>,
}

#[derive(Debug)]
pub struct IconDirectory {
    name: String,
    type_: DirectoryType,
    size: i32,
    scale: i32,
}

#[derive(Debug)]
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
            Some(unknown) => {
                println!("==========> {}", unknown);
            },
            None => {},
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

impl IconTheme {
    pub fn from_dir<T: AsRef<Path>>(path: T) -> Result<IconTheme, ()> {
        let f = Ini::load_from_file(path.as_ref().join("index").with_extension("theme")).map_err(|_| ())?;

        let mut r = Self { base_dirs: vec![path.as_ref().into()], ..Default::default() };
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

    pub fn from_name<T: AsRef<str>>(name: T) -> Result<IconTheme, ()> {

        let system_dir = if cfg!(test) {
            Path::new("tests/icons").join(name.as_ref())
        } else {
            Path::new("/usr/share/icons").join(name.as_ref())
        };

        let user_dirs: Vec<PathBuf> =
            if cfg!(test) {
                  get_user_icon_dir()
                    .iter()
                    .map(|x| x.join(name.as_ref()))
                    .filter(|x| x.is_dir())
                    .collect()
            } else {
                  USER_ICON_DIR
                    .iter()
                    .map(|x| x.join(name.as_ref()))
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
            println!("{:?}", dir);
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

    pub fn append_extra_lookup_dir<T: AsRef<Path>>(&mut self, path: T) {

        self.extra_dirs.push(path.as_ref().into());
    }

    pub fn parents(&self) -> &Vec<String> {
        &self.inherits
    }

    pub fn lookup_icon(&self, name: &IconName, size: i32, scale: i32) -> Option<PathBuf> {

        // find in normal dirs
        for subdir in &self.sub_dirs {
            if !subdir.matches_size(size, scale) { continue; }

            for basedir in &self.base_dirs {
                for ext in EXTS {
                    let p = basedir.join(&subdir.name)
                                   .join(&name.name())
                                   .with_extension(&ext);

                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }

        // test closest file
        let mut minimal_distance = i32::max_value();
        let mut closest_file: Option<PathBuf> = None;

        'dir: for subdir in &self.sub_dirs {
            let distance = subdir.size_distance(size, scale);
            if distance >= minimal_distance { continue; }

            'location: for basedir in &self.base_dirs {
                'ext: for ext in EXTS {
                    let p = basedir.join(&subdir.name)
                                   .join(&name.name())
                                   .with_extension(&ext);

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
            for ext in EXTS {
                let p = extra_dir.join(&name.name()).with_extension(&ext);
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
                    for ext in EXTS {
                        let p = basedir.join(&subdir.name)
                                       .join(&fallback.name())
                                       .with_extension(&ext);

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

        let theme = IconTheme::from_name("themed").unwrap();

        assert_eq!(theme.lookup_icon(&"deepin-deb-installer".into(), 32, 1),
                    Some("tests/icons/themed/apps/32/deepin-deb-installer.svg".into()));
    }

    #[test]
    fn test_in_another_base_dir() {
        env::set_var("XDG_DATA_DIRS", "tests:tests/fake_home/.local/share");

        let theme = IconTheme::from_name("themed").unwrap();

        assert_eq!(theme.lookup_icon(&"just-in-another-base".into(), 16, 1),
                    Some("tests/fake_home/.local/share/icons/themed/apps/16/just-in-another-base.png".into()));
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
}
