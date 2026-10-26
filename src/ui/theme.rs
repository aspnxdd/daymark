use freya::prelude::*;

pub const COLORS: ColorsSheet = ColorsSheet {
    primary: Color::from_rgb(177, 207, 161),
    secondary: Color::from_rgb(66, 91, 65),
    tertiary: Color::from_rgb(31, 47, 37),
    background: Color::from_rgb(16, 22, 19),
    surface_primary: Color::from_rgb(27, 36, 30),
    surface_secondary: Color::from_rgb(23, 31, 26),
    surface_tertiary: Color::from_rgb(20, 28, 23),
    border: Color::from_rgb(45, 58, 48),
    border_focus: Color::from_rgb(177, 207, 161),
    text_primary: Color::from_rgb(229, 233, 222),
    text_secondary: Color::from_rgb(160, 174, 158),
    text_placeholder: Color::from_rgb(133, 151, 133),
    text_inverse: Color::from_rgb(19, 31, 21),
    text_highlight: Color::from_rgb(59, 87, 64),
    focus: Color::from_rgb(42, 59, 45),
    active: Color::from_rgb(37, 53, 41),
    ..DARK_COLORS
};

pub fn daymark_theme() -> Theme {
    let mut theme = dark_theme();
    theme.name = "daymark";
    theme.colors = COLORS;
    theme
}
