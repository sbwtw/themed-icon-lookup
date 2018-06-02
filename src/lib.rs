//!
//! This crate can help you find a themed icon.
//!
//! _See_:
//! [Icon lookup specific](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#icon_lookup)
//!

extern crate ini;
extern crate lru_cache;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rayon;
extern crate gtk_icon_cache;

#[macro_use]
pub mod macros;
mod icon_theme;
pub mod ffi;
pub mod icon_lookup;