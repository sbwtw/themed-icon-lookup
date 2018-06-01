
#[cfg(test)]
macro_rules! test_lookup {
    ($theme: expr, $icon: expr, $size: expr, $scale: expr => $want: expr) => {
        let result = lookup!($theme, $icon, $size, $scale);
        let want = Some($want.into());

        assert_eq!(result, want);
    };
}

#[macro_export]
macro_rules! lookup {
    ($theme: expr, $icon: expr, $size: expr, $scale: expr) => {
        find_icon_with_theme_name($theme, $icon, $size, $scale)
    };
}