use std::{cell::RefCell, collections::HashMap, time::Duration};

use freya_testing::prelude::*;

use super::*;

fn contrast(foreground: Color, background: Color) -> f64 {
    fn luminance(color: Color) -> f64 {
        let channels = [color.r(), color.g(), color.b()].map(|channel| {
            let value = f64::from(channel) / 255.;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
    }
    let a = luminance(foreground);
    let b = luminance(background);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn exactly_five_distinct_palettes_have_readable_text_and_focus() {
    assert_eq!(ThemeId::ALL.len(), 5);
    assert_eq!(ThemeId::default(), ThemeId::Sage);
    let test = launch_test(rect);
    for (index, id) in ThemeId::ALL.into_iter().enumerate() {
        let colors = id.colors();
        for other in &ThemeId::ALL[..index] {
            assert_ne!(id.key(), other.key());
            assert_ne!(colors.background, other.colors().background);
        }
        for surface in [
            colors.background,
            colors.surface_primary,
            colors.surface_secondary,
            colors.surface_tertiary,
            colors.active,
        ] {
            for text in [colors.text_primary, colors.text_secondary] {
                assert!(
                    contrast(text, surface) >= 4.5,
                    "{id:?}: text contrast {}",
                    contrast(text, surface)
                );
            }
        }
        for surface in [
            colors.background,
            colors.surface_primary,
            colors.surface_secondary,
            colors.surface_tertiary,
        ] {
            assert!(
                contrast(colors.text_placeholder, surface) >= 4.5,
                "{id:?}: placeholder contrast"
            );
            assert!(
                contrast(colors.error, surface) >= 4.5,
                "{id:?}: error contrast"
            );
            assert!(
                contrast(colors.border_focus, surface) >= 3.,
                "{id:?}: focus contrast"
            );
        }
        assert!(
            contrast(colors.text_primary, colors.text_highlight) >= 4.5,
            "{id:?}: selected editor text contrast"
        );
        assert!(
            contrast(colors.text_inverse, colors.primary) >= 4.5,
            "{id:?}: primary button contrast"
        );
        let theme = test.run_in(|| id.theme());
        let mut select = theme
            .get::<SelectThemePreference>("select")
            .unwrap()
            .clone();
        let select = select.resolve(&colors);
        assert_eq!(select.color, colors.text_primary);
        assert_eq!(select.background_button, colors.surface_tertiary);
        assert_eq!(select.focus_border_fill, colors.border_focus);
        let mut menu = theme
            .get::<MenuItemThemePreference>("menu_item")
            .unwrap()
            .clone();
        let menu = menu.resolve(&colors);
        assert!(
            contrast(menu.select_color, menu.select_background) >= 4.5,
            "{id:?}: selected menu contrast"
        );
        let mut flat = theme
            .get::<ButtonColorsThemePreference>("flat_button")
            .unwrap()
            .clone();
        assert_eq!(flat.resolve(&colors).focus_border_fill, colors.border_focus);
    }
}

#[test]
fn preference_defaults_without_a_file_and_round_trips_all_themes() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    let mut preference = ThemePreference::load(&database);
    assert_eq!(preference.selected, ThemeId::Sage);
    assert!(preference.error.is_none());
    assert!(!preference.path.exists());
    for id in ThemeId::ALL {
        preference.select(id);
        assert!(preference.error.is_none());
        assert_eq!(
            fs::read_to_string(&preference.path).unwrap(),
            format!("{}\n", id.key())
        );
        let reopened = ThemePreference::load(&database);
        assert_eq!(reopened.selected, id);
        assert!(reopened.error.is_none());
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&preference.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        1,
        "no temporary preference files left behind"
    );
}

#[test]
fn malformed_preferences_fall_back_without_touching_journal_data() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    fs::write(&database, "fictional database sentinel").unwrap();
    for value in [
        b"unknown".to_vec(),
        Vec::new(),
        vec![b'x'; 4096],
        vec![0xff],
    ] {
        fs::write(database.with_extension("theme"), &value).unwrap();
        let mut preference = ThemePreference::load(&database);
        assert_eq!(preference.selected, ThemeId::Sage);
        assert!(preference.error.is_some());
        assert_eq!(
            fs::read(&preference.path).unwrap(),
            value,
            "loading must not replace an invalid preference"
        );
        preference.select(ThemeId::Ocean);
        assert!(preference.error.is_none());
        assert_eq!(ThemePreference::load(&database).selected, ThemeId::Ocean);
        assert_eq!(
            fs::read_to_string(&database).unwrap(),
            "fictional database sentinel"
        );
    }
}

#[test]
fn failed_preference_save_keeps_session_selection_and_supports_retry() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    fs::create_dir(database.with_extension("theme")).unwrap();
    let mut preference = ThemePreference::load(&database);
    assert!(preference.error.is_some());
    preference.select(ThemeId::Paper);
    assert_eq!(preference.selected, ThemeId::Paper);
    assert!(preference.error.is_some());
    fs::remove_dir(&preference.path).unwrap();
    preference.select(ThemeId::Paper);
    assert!(preference.error.is_none());
    assert_eq!(ThemePreference::load(&database).selected, ThemeId::Paper);
}

#[cfg(unix)]
#[test]
fn preference_rejects_symlinks_without_overwriting_the_target() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    let target = dir.path().join("untouched.txt");
    fs::write(&target, "fictional private sentinel").unwrap();
    std::os::unix::fs::symlink(&target, database.with_extension("theme")).unwrap();
    let mut preference = ThemePreference::load(&database);
    assert!(preference.error.is_some());
    preference.select(ThemeId::Dusk);
    assert!(preference.error.is_some());
    assert_eq!(
        fs::read_to_string(target).unwrap(),
        "fictional private sentinel"
    );
}

fn selector_fixture(database: &Path) -> TestingRunner {
    let initial = RefCell::new(Some(ThemePreference::load(database)));
    let (mut test, _) = TestingRunner::new(
        move || {
            let preference = use_state(|| initial.borrow_mut().take().unwrap());
            use_provide_context(|| preference);
            let theme = use_init_theme(|| preference.peek().selected.theme());
            let name = theme.read().name;
            rect()
                .width(Size::px(300.))
                .spacing(8.)
                .child(ThemeSelector)
                .child(label().text(format!("Active palette: {name}")))
        },
        Size2D::new(500., 500.),
        |_| {},
        1.,
    );
    test.set_fonts(HashMap::from([(
        "DM Sans",
        include_bytes!("../../../assets/fonts/DMSans.ttf").as_slice(),
    )]));
    test.set_default_fonts(&["DM Sans".into()]);
    settle(&mut test);
    test
}

fn settle(test: &mut TestingRunner) {
    test.sync_and_update();
    test.poll_n(Duration::from_millis(5), 30);
    test.sync_and_update();
}

fn label_exists(test: &TestingRunner, text: &str) -> bool {
    test.find(|node, element| {
        Label::try_downcast(element)
            .filter(|label| label.text == text)
            .map(|_| node)
    })
    .is_some()
}

fn click_label(test: &mut TestingRunner, text: &str) {
    let node = test
        .find(|node, element| {
            Label::try_downcast(element)
                .filter(|label| label.text == text)
                .map(|_| node)
        })
        .unwrap_or_else(|| panic!("label not found: {text}"));
    let center = node.layout().area.center();
    test.click_cursor((f64::from(center.x), f64::from(center.y)));
    settle(test);
}

#[test]
fn native_selector_changes_and_restores_theme_with_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    let mut test = selector_fixture(&database);
    assert!(label_exists(&test, "Theme: Sage"));
    click_label(&mut test, "Theme: Sage");
    for id in ThemeId::ALL {
        assert!(label_exists(
            &test,
            if id == ThemeId::Sage {
                "Sage · selected"
            } else {
                id.name()
            }
        ));
    }
    click_label(&mut test, "Paper");
    assert!(label_exists(&test, "Theme: Paper"));
    assert!(label_exists(&test, "Active palette: paper"));
    assert_eq!(ThemePreference::load(&database).selected, ThemeId::Paper);
    drop(test);
    let reopened = selector_fixture(&database);
    assert!(label_exists(&reopened, "Theme: Paper"));
    assert!(label_exists(&reopened, "Active palette: paper"));
}

#[test]
fn native_selector_supports_keyboard_selection_and_escape() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    let mut test = selector_fixture(&database);
    test.press_key(Key::Named(NamedKey::Tab));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::Enter));
    settle(&mut test);
    assert!(label_exists(&test, "Ocean"));
    test.press_key(Key::Named(NamedKey::ArrowDown));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::Escape));
    settle(&mut test);
    assert!(!label_exists(&test, "Ocean"));
    test.press_key(Key::Named(NamedKey::Enter));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::ArrowDown));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::ArrowDown));
    settle(&mut test);
    test.press_key(Key::Named(NamedKey::Enter));
    settle(&mut test);
    assert!(label_exists(&test, "Theme: Ocean"));
    assert!(label_exists(&test, "Active palette: ocean"));
    assert_eq!(ThemePreference::load(&database).selected, ThemeId::Ocean);
    test.press_key(Key::Named(NamedKey::Escape));
    settle(&mut test);
    assert!(!label_exists(&test, "Ocean · selected"));
    test.press_key(Key::Named(NamedKey::Enter));
    settle(&mut test);
    assert!(label_exists(&test, "Ocean · selected"));
}

#[test]
fn native_selector_reports_save_failure_without_reverting_palette() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("journal.sqlite3");
    let mut test = selector_fixture(&database);
    fs::create_dir(database.with_extension("theme")).unwrap();
    click_label(&mut test, "Theme: Sage");
    click_label(&mut test, "Ember");
    assert!(label_exists(&test, "Theme: Ember"));
    assert!(label_exists(&test, "Active palette: ember"));
    assert!(
        test.find(|node, element| Paragraph::try_downcast(element)
            .filter(|paragraph| paragraph
                .spans
                .iter()
                .any(|span| span.text.contains("could not be saved")))
            .map(|_| node))
            .is_some()
    );
    fs::remove_dir(database.with_extension("theme")).unwrap();
    click_label(&mut test, "Theme: Ember");
    click_label(&mut test, "Ember · selected");
    assert_eq!(ThemePreference::load(&database).selected, ThemeId::Ember);
}
