
use icon_theme::*;

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

macro_rules! ret_if_found {
    ($value: expr) => {
        if let Some(path) = $value {
            assert!(path.is_file());
            return Some(path)
        }
    };
}

lazy_static! {
    static ref HICOLOR_THEME: Option<Arc<IconTheme>> = IconTheme::from_name("hicolor").ok();
    static ref DEFAULT_THEME_NAME: String = get_default_icon_theme_name().unwrap_or("hicolor".to_string());
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

    let theme = IconTheme::from_name(theme.as_ref()).ok();
    let icon = &icon.into();

    if let Some(ref theme) = theme {
        ret_if_found!(theme.lookup_icon(icon, size, scale));

        // find in parents
        for parent in theme.parents() {
            if let Ok(parent_theme) = IconTheme::from_name(parent) {
                ret_if_found!(parent_theme.lookup_icon(icon, size, scale));
            }
        }
    }

    // find in hicolor
    if let Some(ref hicolor) = *HICOLOR_THEME {
        ret_if_found!(hicolor.lookup_icon(icon, size, scale));
    }

    // fallback
    if let Some(ref theme) = theme {
        ret_if_found!(theme.lookup_fallback_icon(icon, size, scale));

        // fallback in parents
        for parent in theme.parents() {
            if let Ok(parent_theme) = IconTheme::from_name(parent) {
                ret_if_found!(parent_theme.lookup_fallback_icon(icon, size, scale));
            }
        }
    }

    // fallback in hicolor
    if let Some(ref hicolor) = *HICOLOR_THEME {
        ret_if_found!(hicolor.lookup_fallback_icon(icon, size, scale));
    }

    None
}

pub fn find_icon<I>(icon: I, size: i32, scale: i32) -> Option<PathBuf>
  where I: AsRef<str> {

    lookup!(&*DEFAULT_THEME_NAME, icon, size, scale)
}

#[cfg(test)]
mod test {
    use icon_lookup::*;

    use std::env;

    #[test]
    fn test_find_fixed() {
        env::set_var("XDG_DATA_DIRS", "tests");

        test_lookup!("themed", "deepin-deb-installer", 16, 1
                    => "tests/icons/themed/apps/16/deepin-deb-installer.svg");
        test_lookup!("themed", "deepin-deb-installer", 32, 1
                    => "tests/icons/themed/apps/32/deepin-deb-installer.svg");
        test_lookup!("themed", "deepin-deb-installer", 48, 1
                    => "tests/icons/themed/apps/48/deepin-deb-installer.svg");
        test_lookup!("themed", "deepin-deb-installer", 96, 1
                    => "tests/icons/themed/apps/scalable/deepin-deb-installer.svg");
        test_lookup!("themed", "deepin-deb-installer", 24, 1
                    => "tests/icons/themed/apps/scalable/deepin-deb-installer.svg");
    }

    #[test]
    fn test_invalid_theme_name() {
        env::set_var("XDG_DATA_DIRS", "tests");

        // should be fallback to hicolor
        assert_eq!(Some("tests/icons/hicolor/apps/16/TestAppIcon.png".into()),
                    find_icon_with_theme_name("InvalidThemeName", "TestAppIcon", 16, 1));
    }
}