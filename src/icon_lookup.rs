
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

    let theme = IconTheme::from_name(theme.as_ref()).unwrap();
    let icon = &icon.into();

    ret_if_found!(theme.lookup_icon(icon, size, scale));

    unimplemented!()
}

// pub fn find_icon_with_theme<I>(theme: &IconTheme, icon: &IconName, size: i32, scale: i32) -> Option<PathBuf>

    // find in theme
    // ret_if_found!(theme.lookup_icon(icon, size, scale));
//
    // unimplemented!()
// }