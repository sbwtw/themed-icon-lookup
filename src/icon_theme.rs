
use ini::Ini;

use std::path::{Path, PathBuf};
use std::convert::From;
use std::env;

static EXTS: &'static [&'static str] = &["png", "svg"];

lazy_static!{
    static ref USER_ICON_DIR: Option<PathBuf> = get_user_icon_dir();
}

fn get_user_icon_dir() -> Option<PathBuf> {
    env::var("XDG_DATA_HOME")
        .or(env::var("HOME").map(|x| format!("{}/.local/share", x)))
        .map(|x| x.into())
        .ok()
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

#[derive(Debug)]
struct IconThemeLocation {
    base_dir: PathBuf,
    sub_dirs: Vec<IconDirectory>,
}

#[derive(Debug, Default)]
pub struct IconTheme {
    name: String,
    inherits: Vec<String>,
    locations: Vec<IconThemeLocation>,
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
        let f = Ini::load_from_file(path.as_ref().join("index").with_extension("theme"));
        if f.is_err() { return Err(()); }
        let f = f.unwrap();

        let mut r = Self { ..Default::default() };
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

        let dirs = directories.iter().map(
            |x| IconDirectory::with_settings(&f, x)
        ).collect();

        r.locations.push(IconThemeLocation{ base_dir: path.as_ref().into(), sub_dirs: dirs });

        Ok(r)
    }

    pub fn from_name<T: AsRef<str>>(name: T) -> Result<IconTheme, ()> {

        Self::from_system_path(name)
    }

    pub fn from_user_path<T: AsRef<str>>(name: T) -> Result<IconTheme, ()> {

       match &*USER_ICON_DIR {
            Some(user_dir) => Self::from_dir(Path::new(user_dir).join(name.as_ref())),
            None => Err(()),
        }
    }

    pub fn from_system_path<T: AsRef<str>>(name: T) -> Result<IconTheme, ()> {

        let p = if cfg!(test) {
            Path::new("tests/icons").join(name.as_ref())
        } else {
            Path::new("/usr/share/icons").join(name.as_ref())
        };

        Self::from_dir(p)
    }

    pub fn parents(&self) -> &Vec<String> {
        &self.inherits
    }

    pub fn lookup_icon(&self, name: &IconName, size: i32, scale: i32) -> Option<PathBuf> {

        // find in normal dirs
        for location in &self.locations {
            for subdir in &location.sub_dirs {
                if !subdir.matches_size(size, scale) { continue; }

                for ext in EXTS {
                    let p = location.base_dir
                                .join(&subdir.name)
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

        'location: for location in &self.locations {
            'dir: for subdir in &location.sub_dirs {
                let distance = subdir.size_distance(size, scale);
                if distance >= minimal_distance { continue; }

                'ext: for ext in EXTS {
                    let p = location.base_dir
                                .join(&subdir.name)
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

        closest_file
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
            for location in &self.locations {
                for subdir in &location.sub_dirs {
                    for ext in EXTS {
                        let p = location.base_dir
                                    .join(&subdir.name)
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

        assert_eq!(get_user_icon_dir(), None);

        env::set_var("HOME", "fake_home");
        assert_eq!(get_user_icon_dir(), Some("fake_home/.local/share".into()));

        env::set_var("XDG_DATA_HOME", "fake_xdg_home/.local/share");
        assert_eq!(get_user_icon_dir(), Some("fake_xdg_home/.local/share".into()));
    }

    #[test]
    fn test_icon_name_fallback() {
        let mut icon_name = IconName::from("some-icon-name.svg");

        println!("{:?}", icon_name);
        while let Some(icon_name) = icon_name.fallback() {
            println!("{:?}", icon_name);
        }
    }

    #[test]
    fn test_lookup_threshold() {
        let theme = IconTheme::from_name("hicolor").unwrap();

        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 46, 1),
                    Some("tests/icons/hicolor/apps/48/TestAppIcon.png".into()));
        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 50, 1),
                    Some("tests/icons/hicolor/apps/48/TestAppIcon.png".into()));
        assert_eq!(theme.lookup_icon(&"TestAppIcon".into(), 51, 1),
                    Some("tests/icons/hicolor/apps/scalable/TestAppIcon.svg".into()));
    }
}
