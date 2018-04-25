
use ini::Ini;

use std::path::{Path, PathBuf};
use std::convert::{From, Into};

static EXTS: &'static [&'static str] = &["png", "svg"];

#[derive(Debug)]
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
}

#[derive(Debug, Default)]
pub struct IconTheme {
    name: String,
    basedir: String,
    inherits: Option<Vec<String>>,
    directories: Vec<IconDirectory>,
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

        let properties = settings.section(Some(name.as_ref())).unwrap();

        let mut r = Self {
            name: name.as_ref().to_string(),
            ..Default::default()
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
                let min = properties.get("MinSize").map(|x| x.parse().unwrap_or(r.size));
                let max = properties.get("MaxSize").map(|x| x.parse().unwrap_or(r.size));

                r.type_ = DirectoryType::Scalable(min.unwrap_or(r.size), max.unwrap_or(r.size))
            },
            Some("Threshold") => {
                r.type_ = DirectoryType::Threshold(properties.get("Threshold").unwrap().parse().unwrap_or(2));
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
            DirectoryType::Threshold(threshold) => size - threshold <= size && size + threshold >= size,
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
    pub fn from_file<T: AsRef<Path>>(file: T) -> Result<IconTheme, ()> {
        let f = Ini::load_from_file(file.as_ref());
        if f.is_err() { return Err(()); }
        let f = f.unwrap();

        let mut r = Self { ..Default::default() };
        let mut directories = vec![];

        if let Some(properties) = f.section(Some("Icon Theme")) {
            r.name = properties.get("Name").unwrap_or(&String::new()).to_string();

            if let Some(list) = properties.get("Inherits").map(|x| x.split(',')) {
                let inherits: Vec<String> = list.map(|x| x.to_string()).collect();
                r.inherits = Some(inherits);
            }

            if let Some(list) = properties.get("Directories").map(|x| x.split(',')) {
                directories = list.map(|x| x.to_string()).collect();
            }
        };

        r.directories = directories.iter().map(
            |x| IconDirectory::with_settings(&f, x)
        ).collect();

        Ok(r)
    }

    pub fn lookup_icon(&self, name: &IconName, size: i32, scale: i32) -> Option<PathBuf> {

        let name: &str = name.name();
        let path = Path::new("/usr/share/icons/Flattr");

        for subdir in &self.directories {
            if !subdir.matches_size(size, scale) { continue; }

            for ext in EXTS {
                let p = path.join(&subdir.name)
                            .join(&name)
                            .with_extension(&ext);

                if p.is_file() {
                    return Some(p);
                }
            }
        }

        None
    }

    pub fn lookup_fallback_icon(&self, name: &IconName) -> Option<PathBuf> {

        None
    }
}

#[cfg(test)]
mod test {
    use icon_theme::*;

    #[test]
    fn test_icon_theme() {
        let f = "/usr/share/icons/Flattr/index.theme";
        let icon_theme = IconTheme::from_file(f).unwrap();

        let r = icon_theme.lookup_icon(&"system-suspend".into(), 32, 1);
        println!("{:#?}", r);
    }
}