
#[cfg(test)]
macro_rules! test_lookup {

    ($icon: expr, $size: expr, $scale: expr => $($wants: expr),+) => {
        let result = find_icon($icon, $size, $scale).unwrap();

        let mut availables: HashSet<PathBuf> = HashSet::new();
        $(
            availables.insert($wants.into());
        )+

        if !availables.contains(&result) {
            panic!("\nresult: {:?},\nwants: {:?}", result, availables);
        }
    };

    ($theme: ident, $icon: expr, $size: expr, $scale: expr => $($wants: expr),+) => {
        let result = find_icon_in_theme(&$theme, $icon, $size, $scale).unwrap();

        let mut availables: HashSet<PathBuf> = HashSet::new();
        $(
            availables.insert($wants.into());
        )+

        if !availables.contains(&result) {
            panic!("\nresult: {:?},\nwants: {:?}", result, availables);
        }
    };

    ($theme: expr, $icon: expr, $size: expr, $scale: expr => $($wants: expr),+) => {{
        let result = lookup!($theme, $icon, $size, $scale).unwrap();

        let mut availables: HashSet<PathBuf> = HashSet::new();
        $(
            availables.insert($wants.into());
        )+

        if !availables.contains(&result) {
            panic!("result: {:?},\nwants: {:?}", result, availables);
        }
    }};
}

#[macro_export]
macro_rules! lookup {
    ($theme: expr, $icon: expr, $size: expr, $scale: expr) => {
        find_icon_with_theme_name($theme, $icon, $size, $scale)
    };
}