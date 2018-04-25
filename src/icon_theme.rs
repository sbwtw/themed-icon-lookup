
use ini::Ini;

use std::path::Path;

#[derive(Debug, Default)]
struct IconTheme {
    name: String,
    inherits: Option<Vec<String>>,
    directories: Vec<IconDirectory>,
}

#[derive(Debug, Default)]
struct IconDirectory {
    name: String,
    size: u32,
}

impl IconDirectory {
    fn with_settings<T: AsRef<str>>(settings: &Ini, name: T) -> Self {

        let properties = settings.section(Some(name.as_ref())).unwrap();

        Self {
            name: name.as_ref().to_string(),
            size: properties.get("Size").unwrap().parse().unwrap(),
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
}

#[cfg(test)]
mod test {
    use icon_theme::*;

    #[test]
    fn test_icon_theme() {
        let f = "/usr/share/icons/Flattr/index.theme";
        let icon_theme = IconTheme::from_file(f).unwrap();

        println!("{:#?}", icon_theme);
    }
}