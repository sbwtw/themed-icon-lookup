//!
//! This crate can help you find a themed icon.
//!
//! _See_:
//! [Icon lookup specific](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#icon_lookup)
//!

mod icon_theme;

use std::path::PathBuf;

macro_rules! ret_if_found {
    ($value: expr) => {
        if let Some(path) = $value {
            if path.is_file() {
                return Some(path)
            }
        }
    };
}

pub fn find_icon_with_theme<T, I>(theme: T, icon: I, size: u32, scale: f32) -> Option<PathBuf>
  where T: AsRef<str>, I: AsRef<str> {

    // find in theme
    ret_if_found!(lookup_icon(theme, icon, size, scale));

    unimplemented!()
}

fn lookup_icon<T, I>(theme: T, icon: I, size: u32, scale: f32) -> Option<PathBuf>
  where T: AsRef<str>, I: AsRef<str> {

    unimplemented!()
}

fn lookup_fallback_icon<T: AsRef<str>>(icon: T) -> Option<PathBuf> {
    unimplemented!()
}