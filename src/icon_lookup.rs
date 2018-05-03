
use icon_theme::*;

use std::path::PathBuf;

macro_rules! ret_if_found {
    ($value: expr) => {
        if let Some(path) = $value {
            assert!(path.is_file());
            return Some(path)
        }
    };
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

    // TODO: get default icon theme
    find_icon_with_theme_name("Flattr", icon, size, scale)
}