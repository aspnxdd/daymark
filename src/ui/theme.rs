use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use freya::prelude::*;

// Sage is also the native window's bootstrap background before the app renders.
pub const COLORS: ColorsSheet = ColorsSheet {
    primary: Color::from_rgb(177, 207, 161),
    secondary: Color::from_rgb(66, 91, 65),
    tertiary: Color::from_rgb(31, 47, 37),
    background: Color::from_rgb(16, 22, 19),
    surface_primary: Color::from_rgb(27, 36, 30),
    surface_secondary: Color::from_rgb(23, 31, 26),
    surface_tertiary: Color::from_rgb(20, 28, 23),
    surface_inverse: Color::from_rgb(133, 151, 133),
    surface_inverse_secondary: Color::from_rgb(160, 174, 158),
    surface_inverse_tertiary: Color::from_rgb(177, 207, 161),
    border: Color::from_rgb(45, 58, 48),
    border_focus: Color::from_rgb(177, 207, 161),
    border_disabled: Color::from_rgb(45, 58, 48),
    text_primary: Color::from_rgb(229, 233, 222),
    text_secondary: Color::from_rgb(160, 174, 158),
    text_placeholder: Color::from_rgb(133, 151, 133),
    text_inverse: Color::from_rgb(19, 31, 21),
    text_highlight: Color::from_rgb(59, 87, 64),
    focus: Color::from_rgb(42, 59, 45),
    active: Color::from_rgb(37, 53, 41),
    disabled: Color::from_rgb(31, 40, 34),
    success: Color::from_rgb(177, 207, 161),
    warning: Color::from_rgb(239, 203, 139),
    error: Color::from_rgb(255, 180, 165),
    info: Color::from_rgb(163, 199, 218),
    ..DARK_COLORS
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeId {
    #[default]
    Sage,
    Ocean,
    Dusk,
    Ember,
    Paper,
}

impl ThemeId {
    pub const ALL: [Self; 5] = [
        Self::Sage,
        Self::Ocean,
        Self::Dusk,
        Self::Ember,
        Self::Paper,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sage => "Sage",
            Self::Ocean => "Ocean",
            Self::Dusk => "Dusk",
            Self::Ember => "Ember",
            Self::Paper => "Paper",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Sage => "sage",
            Self::Ocean => "ocean",
            Self::Dusk => "dusk",
            Self::Ember => "ember",
            Self::Paper => "paper",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.key() == value.trim())
    }

    pub fn colors(self) -> ColorsSheet {
        match self {
            Self::Sage => COLORS,
            Self::Ocean => ColorsSheet {
                primary: Color::from_rgb(151, 205, 222),
                secondary: Color::from_rgb(57, 91, 106),
                tertiary: Color::from_rgb(28, 46, 58),
                background: Color::from_rgb(15, 22, 29),
                surface_primary: Color::from_rgb(27, 38, 49),
                surface_secondary: Color::from_rgb(22, 32, 42),
                surface_tertiary: Color::from_rgb(19, 28, 37),
                surface_inverse: Color::from_rgb(139, 157, 172),
                surface_inverse_secondary: Color::from_rgb(165, 184, 198),
                surface_inverse_tertiary: Color::from_rgb(151, 205, 222),
                border: Color::from_rgb(45, 61, 76),
                border_focus: Color::from_rgb(151, 205, 222),
                border_disabled: Color::from_rgb(45, 61, 76),
                text_primary: Color::from_rgb(226, 234, 240),
                text_secondary: Color::from_rgb(165, 184, 198),
                text_placeholder: Color::from_rgb(139, 157, 172),
                text_inverse: Color::from_rgb(14, 32, 42),
                text_highlight: Color::from_rgb(43, 75, 96),
                focus: Color::from_rgb(32, 53, 69),
                active: Color::from_rgb(30, 48, 63),
                disabled: Color::from_rgb(28, 39, 50),
                success: Color::from_rgb(167, 211, 184),
                info: Color::from_rgb(151, 205, 222),
                ..COLORS
            },
            Self::Dusk => ColorsSheet {
                primary: Color::from_rgb(205, 182, 231),
                secondary: Color::from_rgb(85, 69, 108),
                tertiary: Color::from_rgb(46, 34, 60),
                background: Color::from_rgb(23, 18, 29),
                surface_primary: Color::from_rgb(39, 31, 48),
                surface_secondary: Color::from_rgb(33, 26, 41),
                surface_tertiary: Color::from_rgb(29, 23, 36),
                surface_inverse: Color::from_rgb(160, 145, 175),
                surface_inverse_secondary: Color::from_rgb(185, 170, 199),
                surface_inverse_tertiary: Color::from_rgb(205, 182, 231),
                border: Color::from_rgb(61, 49, 73),
                border_focus: Color::from_rgb(205, 182, 231),
                border_disabled: Color::from_rgb(61, 49, 73),
                text_primary: Color::from_rgb(236, 228, 241),
                text_secondary: Color::from_rgb(185, 170, 199),
                text_placeholder: Color::from_rgb(160, 145, 175),
                text_inverse: Color::from_rgb(33, 22, 45),
                text_highlight: Color::from_rgb(77, 52, 95),
                focus: Color::from_rgb(55, 40, 70),
                active: Color::from_rgb(50, 36, 64),
                disabled: Color::from_rgb(41, 32, 50),
                success: Color::from_rgb(182, 208, 177),
                info: Color::from_rgb(184, 191, 235),
                ..COLORS
            },
            Self::Ember => ColorsSheet {
                primary: Color::from_rgb(231, 186, 145),
                secondary: Color::from_rgb(104, 77, 55),
                tertiary: Color::from_rgb(57, 39, 29),
                background: Color::from_rgb(27, 20, 17),
                surface_primary: Color::from_rgb(44, 33, 27),
                surface_secondary: Color::from_rgb(37, 28, 23),
                surface_tertiary: Color::from_rgb(33, 25, 21),
                surface_inverse: Color::from_rgb(171, 149, 133),
                surface_inverse_secondary: Color::from_rgb(195, 173, 155),
                surface_inverse_tertiary: Color::from_rgb(231, 186, 145),
                border: Color::from_rgb(70, 52, 42),
                border_focus: Color::from_rgb(231, 186, 145),
                border_disabled: Color::from_rgb(70, 52, 42),
                text_primary: Color::from_rgb(242, 230, 216),
                text_secondary: Color::from_rgb(195, 173, 155),
                text_placeholder: Color::from_rgb(171, 149, 133),
                text_inverse: Color::from_rgb(43, 26, 16),
                text_highlight: Color::from_rgb(93, 60, 36),
                focus: Color::from_rgb(68, 47, 33),
                active: Color::from_rgb(60, 42, 31),
                disabled: Color::from_rgb(47, 35, 29),
                success: Color::from_rgb(187, 207, 156),
                info: Color::from_rgb(176, 203, 218),
                ..COLORS
            },
            Self::Paper => ColorsSheet {
                primary: Color::from_rgb(57, 91, 65),
                secondary: Color::from_rgb(81, 110, 83),
                tertiary: Color::from_rgb(218, 229, 211),
                background: Color::from_rgb(245, 242, 233),
                surface_primary: Color::from_rgb(235, 232, 220),
                surface_secondary: Color::from_rgb(255, 252, 244),
                surface_tertiary: Color::from_rgb(239, 236, 225),
                surface_inverse: Color::from_rgb(109, 113, 98),
                surface_inverse_secondary: Color::from_rgb(86, 96, 79),
                surface_inverse_tertiary: Color::from_rgb(57, 91, 65),
                border: Color::from_rgb(189, 192, 174),
                border_focus: Color::from_rgb(57, 91, 65),
                border_disabled: Color::from_rgb(189, 192, 174),
                text_primary: Color::from_rgb(43, 48, 39),
                text_secondary: Color::from_rgb(81, 91, 73),
                text_placeholder: Color::from_rgb(99, 107, 91),
                text_inverse: Color::from_rgb(255, 252, 244),
                text_highlight: Color::from_rgb(200, 218, 189),
                focus: Color::from_rgb(219, 230, 211),
                active: Color::from_rgb(224, 230, 216),
                disabled: Color::from_rgb(224, 222, 211),
                success: Color::from_rgb(57, 91, 65),
                warning: Color::from_rgb(116, 77, 13),
                error: Color::from_rgb(153, 52, 41),
                info: Color::from_rgb(45, 87, 119),
                ..LIGHT_COLORS
            },
        }
    }

    pub fn theme(self) -> Theme {
        let mut theme = if self == Self::Paper {
            light_theme()
        } else {
            dark_theme()
        };
        theme.name = self.key();
        theme.colors = self.colors();
        // Freya's flat-button default uses the subtle border token for keyboard focus.
        theme.set(
            "flat_button",
            ButtonColorsThemePreference {
                background: Preference::Specific(Color::TRANSPARENT),
                hover_background: Preference::Reference("surface_tertiary"),
                border_fill: Preference::Specific(Color::TRANSPARENT),
                focus_border_fill: Preference::Reference("border_focus"),
                color: Preference::Reference("text_primary"),
            },
        );
        theme
    }
}

pub(super) struct ThemePreference {
    path: PathBuf,
    pub selected: ThemeId,
    pub error: Option<&'static str>,
}

impl ThemePreference {
    pub fn load(database: &Path) -> Self {
        let path = database.with_extension("theme");
        let mut preference = Self {
            path,
            selected: ThemeId::default(),
            error: None,
        };
        let result = (|| {
            regular_file_or_missing(&preference.path)?;
            let file = File::open(&preference.path)?;
            let mut value = String::new();
            file.take(65).read_to_string(&mut value)?;
            Ok::<_, io::Error>(
                (value.len() <= 64)
                    .then(|| ThemeId::parse(&value))
                    .flatten(),
            )
        })();
        match result {
            Ok(Some(selected)) => preference.selected = selected,
            Ok(None) => {
                preference.error = Some(
                    "Theme preference was not recognized. Using Sage; choose a theme to replace it.",
                )
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {
                preference.error = Some(
                    "Could not read the theme preference. Using Sage; choose a theme to try saving again.",
                )
            }
        }
        preference
    }

    fn select(&mut self, selected: ThemeId) {
        self.selected = selected;
        self.error = self.save().err().map(|_| "Theme works for this session, but the preference could not be saved. Select it again to retry.");
    }

    fn save(&self) -> io::Result<()> {
        regular_file_or_missing(&self.path)?;
        static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
        let temporary = self.path.with_extension(format!(
            "theme.{}.{}.tmp",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(&temporary)?;
        let result = (|| {
            file.write_all(format!("{}\n", self.selected.key()).as_bytes())?;
            file.sync_all()?;
            drop(file);
            regular_file_or_missing(&self.path)?;
            fs::rename(&temporary, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn regular_file_or_missing(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if !meta.file_type().is_file() => {
            Err(io::Error::other("Theme preference is not a regular file"))
        }
        Ok(_) => {
            #[cfg(unix)]
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
            Ok(())
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

/// Native Freya selector; DaymarkApp provides its preference and Theme contexts.
#[derive(PartialEq)]
pub struct ThemeSelector;

impl Component for ThemeSelector {
    fn render(&self) -> impl IntoElement {
        let mut preference = use_consume::<State<ThemePreference>>();
        let mut theme = use_theme();
        let selected = preference.read().selected;
        let error = preference.read().error;
        let colors = theme.read().colors.clone();
        rect()
            .width(Size::fill())
            .spacing(8.)
            .on_key_down(|event: Event<KeyboardEventData>| {
                if event.key == Key::Named(NamedKey::Escape) {
                    // Pinned Select closes on Escape but otherwise drops menu-item focus to root.
                    let group = Platform::get()
                        .current_window()
                        .focused_accessibility_node
                        .peek()
                        .member_of();
                    if let Some(group) = group {
                        group.request_focus();
                    }
                }
            })
            .child(
                Select::new()
                    .width(Size::fill())
                    .selected_item(
                        label()
                            .text(format!("Theme: {}", selected.name()))
                            .font_size(13.),
                    )
                    .children(ThemeId::ALL.into_iter().map(|id| {
                        MenuItem::new()
                            .key(id.key())
                            .selected(id == selected)
                            .on_press(move |_| {
                                preference.write().select(id);
                                theme.set(id.theme());
                            })
                            .child(
                                label()
                                    .text(if id == selected {
                                        format!("{} · selected", id.name())
                                    } else {
                                        id.name().to_string()
                                    })
                                    .font_size(13.),
                            )
                            .into_element()
                    })),
            )
            .maybe_child(error.map(|message| {
                paragraph()
                    .width(Size::fill())
                    .font_size(12.)
                    .color(colors.error)
                    .span(message)
            }))
    }
}

#[cfg(test)]
mod tests;
