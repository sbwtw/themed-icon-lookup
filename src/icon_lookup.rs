
use icon_theme::*;

use std::path::PathBuf;
use std::process::Command;

macro_rules! ret_if_found {
    ($value: expr) => {
        if let Some(path) = $value {
            assert!(path.is_file());
            return Some(path)
        }
    };
}

fn get_default_icon_theme_name() -> Option<String> {
    let result = Command::new("gsettings")
                    .arg("get")
                    .arg("org.gnome.desktop.interface")
                    .arg("icon-theme")
                    .output()
                    .expect("failed to get default icon theme");

    if result.status.success() {
        let ref name = result.stdout[1..result.stdout.len() - 2];
        Some(String::from_utf8_lossy(&name).to_string())
    } else {
        None
    }
}

pub fn find_icon_with_theme_name<T, I>(theme: T, icon: I, size: i32, scale: i32) -> Option<PathBuf>
  where T: AsRef<str>, I: AsRef<str> {

    let theme = IconTheme::from_name(theme.as_ref()).ok()?;
    let icon = &icon.into();

    ret_if_found!(theme.lookup_icon(icon, size, scale));

    // find in parents
    for parent in theme.parents() {
        let parent_theme = IconTheme::from_name(parent).ok()?;

        ret_if_found!(parent_theme.lookup_icon(icon, size, scale));
    }

    // fallback
    ret_if_found!(theme.lookup_fallback_icon(icon, size, scale));

    // fallback in parents
    for parent in theme.parents() {
        let parent_theme = IconTheme::from_name(parent).ok()?;

        ret_if_found!(parent_theme.lookup_fallback_icon(icon, size, scale));
    }

    None
}

pub fn find_icon<I>(icon: I, size: i32, scale: i32) -> Option<PathBuf>
  where I: AsRef<str> {

    let default_theme = get_default_icon_theme_name().unwrap_or("hicolor".to_string());

    find_icon_with_theme_name(default_theme, icon, size, scale)
}

#[cfg(test)]
mod test {
    use icon_lookup::*;

    #[test]
    fn test_default_theme() {
        find_icon("firefox", 48, 1);
    }
}