use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
};

use freya_testing::{TestingNode, prelude::*};

use super::*;
use crate::storage::{Entry, Store};

fn fixture(seed: bool, width: f32, height: f32) -> (tempfile::TempDir, TestingRunner) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    if seed {
        for (date, title, body) in [
            (
                "2026-10-21T09:00:00+00:00",
                "Making a little room",
                "Cleared the desk. Opened the window. Sometimes the smallest changes make the most room.",
            ),
            (
                "2026-10-23T18:30:00+00:00",
                "The long way home",
                "Took the path by the river. The trees are almost all gold now. I want to remember the sound of the leaves under my feet.",
            ),
            (
                "2026-10-25T16:00:00+00:00",
                "Small things, quietly good",
                "The light came through the kitchen window just right this morning. I made coffee slowly, without checking my phone, and watched the steam curl into the air.\n\nNothing extraordinary happened today. A walk around the neighborhood. A few pages of a book. A conversation that made me smile long after it ended.\n\nI’m learning that a good day doesn’t always announce itself. Sometimes it just feels like this: a little softer, a little more my own.\n\nSomething to carry into tomorrow: there is enough time to notice.",
            ),
        ] {
            let mut entry = Entry::blank();
            entry.created_at = date.into();
            entry.title = title.into();
            entry.body = body.into();
            store.save(&mut entry).unwrap();
        }
    }
    let journal = Journal::new(store, Arc::new(AtomicBool::new(false))).unwrap();
    let app = DaymarkApp {
        initial: RefCell::new(Some(journal)),
    };
    let (mut test, _) = TestingRunner::new(
        move || app.render().into_element(),
        Size2D::new(width, height),
        |_| {},
        1.,
    );
    test.set_fonts(HashMap::from([
        (
            "DM Sans",
            include_bytes!("../../assets/fonts/DMSans.ttf").as_slice(),
        ),
        (
            "DM Serif Display",
            include_bytes!("../../assets/fonts/DMSerifDisplay-Regular.ttf").as_slice(),
        ),
    ]));
    test.set_default_fonts(&["DM Sans".into()]);
    (dir, test)
}

fn paragraph_node(test: &TestingRunner, text: &str) -> TestingNode {
    test.find_many(|node, element| {
        Paragraph::try_downcast(element)
            .filter(|p| p.spans.iter().any(|s| s.text == text))
            .map(|_| node)
    })
    .pop()
    .unwrap_or_else(|| panic!("paragraph not found: {text}"))
}

fn click_label(test: &mut TestingRunner, text: &str) {
    let node = test
        .find(|node, element| {
            Label::try_downcast(element)
                .filter(|label| label.text.as_ref() == text)
                .map(|_| node)
        })
        .unwrap_or_else(|| panic!("label not found: {text}"));
    click(test, node);
}

fn click(test: &mut TestingRunner, node: TestingNode) {
    assert!(node.layout().area.height() > 0.);
    let point = node.layout().area.center();
    test.click_cursor((point.x as f64, point.y as f64));
    test.sync_and_update();
    test.render();
}

fn shortcut(test: &mut TestingRunner, key: &str) {
    test.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character(key.into()),
        code: Code::Unidentified,
        modifiers: Modifiers::ctrl_or_meta(),
    });
    test.sync_and_update();
    test.sync_and_update();
    test.render();
}

#[test]
fn sidebar_brand_and_footer_theme_menu_fit_supported_window_sizes() {
    for (width, height) in [(1360., 920.), (1060., 760.)] {
        let (_dir, mut test) = fixture(false, width, height);
        for text in ["Daymark", "Theme: Sage"] {
            let brand = test
                .find(|node, element| {
                    Label::try_downcast(element)
                        .filter(|label| label.text.as_ref() == text)
                        .map(|_| node)
                })
                .expect("Daymark brand is rendered");
            let area = brand.layout().area;
            assert!(area.width() > 0. && area.height() > 0.);
            assert!(area.min_x() >= 0. && area.max_x() <= 280.);
            assert!(area.min_y() >= 0. && area.max_y() <= height);
            if text == "Theme: Sage" {
                assert!(area.min_y() > height - 70.);
            }
        }
        click_label(&mut test, "Theme: Sage");
        test.poll_n(std::time::Duration::from_millis(5), 30);
        for id in theme::ThemeId::ALL {
            let name = if id == theme::ThemeId::Sage {
                "Sage · selected"
            } else {
                id.name()
            };
            let item = test
                .find(|node, element| {
                    Label::try_downcast(element)
                        .filter(|label| label.text.as_ref() == name)
                        .map(|_| node)
                })
                .expect("theme option is rendered above footer");
            let area = item.layout().area;
            assert!(area.min_x() >= 0. && area.max_x() <= 280.);
            assert!(area.min_y() >= 0. && area.max_y() < height - 40.);
        }
    }
}

#[test]
fn writing_search_navigation_delete_and_reopen() {
    let (dir, mut test) = fixture(false, 1360., 920.);
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("First thought 🌿");
    test.press_key(Key::Named(NamedKey::Enter));
    test.write_text("Second line");
    let expected = "First thought 🌿\nSecond line";
    assert!(paragraph_node(&test, expected).layout().area.height() > 0.);
    let store = Store::open(dir.path()).unwrap();
    let entries = store.list("").unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(store.get(entries[0].id).unwrap().body, expected);
    drop(store);

    shortcut(&mut test, "f");
    test.write_text("missing");
    assert!(
        paragraph_node(&test, "No entries found.")
            .layout()
            .area
            .height()
            > 0.
    );
    assert!(paragraph_node(&test, expected).layout().area.height() > 0.);

    shortcut(&mut test, "n");
    assert!(paragraph_node(&test, "Entry text…").layout().area.height() > 0.);
    let previous = paragraph_node(&test, "Untitled entry");
    click(&mut test, previous);
    assert!(paragraph_node(&test, expected).layout().area.height() > 0.);
    click_label(&mut test, "Delete");
    assert!(
        paragraph_node(
            &test,
            "Delete this entry permanently? This cannot be undone."
        )
        .layout()
        .area
        .height()
            > 0.
    );
    click_label(&mut test, "Keep entry");
    assert_eq!(Store::open(dir.path()).unwrap().list("").unwrap().len(), 1);
    click_label(&mut test, "Delete");
    click_label(&mut test, "Delete permanently");
    assert!(
        Store::open(dir.path())
            .unwrap()
            .list("")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn title_focus_mode_and_minimum_window_layout() {
    let (dir, mut test) = fixture(false, 1060., 760.);
    let title = paragraph_node(&test, "Entry title…");
    click(&mut test, title);
    test.write_text("A quiet Sunday");
    shortcut(&mut test, "s");
    assert_eq!(
        Store::open(dir.path()).unwrap().list("").unwrap()[0].title,
        "A quiet Sunday"
    );
    click_label(&mut test, "Focus");
    click_label(&mut test, "Show timeline");
    let title = paragraph_node(&test, "A quiet Sunday");
    click(&mut test, title);
    shortcut(&mut test, "n");
    assert!(paragraph_node(&test, "Entry title…").layout().area.height() > 0.);
}

#[test]
fn undo_is_isolated_between_fields_and_identical_entries() {
    let (dir, mut test) = fixture(false, 1360., 920.);
    let title = paragraph_node(&test, "Entry title…");
    click(&mut test, title);
    test.write_text("Same title");
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("Same body");
    let store = Store::open(dir.path()).unwrap();
    let first = store.list("").unwrap()[0].id;

    shortcut(&mut test, "z");
    assert_eq!(store.get(first).unwrap().body, "");
    assert_eq!(store.get(first).unwrap().title, "Same title");
    let title = paragraph_node(&test, "Same title");
    click(&mut test, title);
    shortcut(&mut test, "z");
    assert_eq!(store.get(first).unwrap().title, "");
    let title = paragraph_node(&test, "Entry title…");
    click(&mut test, title);
    test.write_text("Same title");
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("Same body");
    assert_eq!(store.get(first).unwrap().body, "Same body");

    shortcut(&mut test, "n");
    let title = paragraph_node(&test, "Entry title…");
    click(&mut test, title);
    test.write_text("Same title");
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("Same body");
    let second = store.list("").unwrap()[0].id;
    assert_ne!(first, second);
    let mut rows = test.find_many(|node, element| {
        Paragraph::try_downcast(element)
            .filter(|p| p.spans.iter().any(|s| s.text == "Same title"))
            .map(|_| node)
    });
    assert_eq!(rows.len(), 3);
    click(&mut test, rows.remove(1));
    let body = paragraph_node(&test, "Same body");
    click(&mut test, body);
    shortcut(&mut test, "z");
    assert_eq!(store.get(first).unwrap().body, "Same body");
    test.press_key(Key::Named(NamedKey::End));
    test.write_text("!");
    assert_eq!(store.get(first).unwrap().body, "Same body!");
    assert_eq!(store.get(second).unwrap().body, "Same body");
    shortcut(&mut test, "z");
    assert_eq!(store.get(first).unwrap().body, "Same body");
}

#[test]
fn search_shortcut_restores_timeline_from_focus_mode() {
    let (_dir, mut test) = fixture(false, 1360., 920.);
    click_label(&mut test, "Focus");
    shortcut(&mut test, "f");
    test.write_text("missing");
    assert!(
        paragraph_node(&test, "No entries found.")
            .layout()
            .area
            .height()
            > 0.
    );
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("A private thought");
    assert!(
        paragraph_node(&test, "A private thought")
            .layout()
            .area
            .height()
            > 0.
    );
}

#[test]
fn body_drag_selection_stops_after_release() {
    for release_outside in [false, true] {
        let (_dir, mut test) = fixture(false, 1360., 920.);
        let body = paragraph_node(&test, "Entry text…");
        click(&mut test, body);
        test.write_text("abcdefghij");
        test.render();
        let body = paragraph_node(&test, "abcdefghij");
        let area = body.layout().area;
        let start = (area.min_x() as f64 + 2., area.min_y() as f64 + 15.);
        let end = (start.0 + 50., start.1);
        test.press_cursor(start);
        test.move_cursor(end);
        test.sync_and_update();
        let selection = test
            .find(|_, element| {
                Paragraph::try_downcast(element)
                    .filter(|p| !p.highlights.is_empty())
                    .map(|p| p.highlights.clone())
            })
            .expect("drag should select text");
        assert_ne!(selection[0].0, selection[0].1);
        let release = if release_outside {
            (area.min_x() as f64 - 20., end.1)
        } else {
            end
        };
        test.release_cursor(release);
        test.move_cursor((end.0 + 100., end.1));
        test.sync_and_update();
        let after = test
            .find(|_, element| {
                Paragraph::try_downcast(element)
                    .filter(|p| !p.highlights.is_empty())
                    .map(|p| p.highlights.clone())
            })
            .expect("release should retain selection");
        assert_eq!(after, selection, "released outside: {release_outside}");
    }
}

#[test]
fn themes_preserve_writing_and_undo_and_remember_the_selection() {
    let (dir, mut test) = fixture(false, 1060., 760.);
    let body = paragraph_node(&test, "Entry text…");
    click(&mut test, body);
    test.write_text("A fictional note to keep through five palettes.");
    let store = Store::open(dir.path()).unwrap();
    let entry = store.list("").unwrap()[0].id;
    let mut selected = theme::ThemeId::Sage;
    for next in theme::ThemeId::ALL.into_iter().skip(1) {
        click_label(&mut test, &format!("Theme: {}", selected.name()));
        test.poll_n(std::time::Duration::from_millis(5), 30);
        click_label(&mut test, next.name());
        test.poll_n(std::time::Duration::from_millis(5), 30);
        assert_eq!(
            std::fs::read_to_string(store.path.with_extension("theme"))
                .unwrap()
                .trim(),
            next.name().to_lowercase(),
        );
        assert!(
            paragraph_node(&test, "A fictional note to keep through five palettes.")
                .layout()
                .area
                .height()
                > 0.
        );
        selected = next;
    }
    let body = paragraph_node(&test, "A fictional note to keep through five palettes.");
    click(&mut test, body);
    shortcut(&mut test, "z");
    assert_eq!(store.get(entry).unwrap().body, "");
    assert!(paragraph_node(&test, "Entry text…").layout().area.height() > 0.);
}

#[test]
fn decorative_copy_is_removed_but_privacy_information_remains_accessible() {
    for seed in [false, true] {
        let (_dir, mut test) = fixture(seed, 1060., 760.);
        click_label(&mut test, "Privacy and storage");
        assert!(
            test.find(|_, element| Paragraph::try_downcast(element)
                .filter(|p| p.spans.iter().any(|s| s.text.contains("NOT encrypted")))
                .map(|_| ()))
                .is_some()
        );
        for removed in [
            "SLOW DOWN. NOTICE MORE.",
            "Keep it here.",
            "A LITTLE SPACE FOR YOUR DAYS",
            "Just you. Just this device.",
            "DAYMARK / LOCAL JOURNAL",
            "Your story starts here.",
            "Write your first entry. It will find a home in this timeline.",
            "A new page in your story",
            "A fresh page",
            "A moment kept",
            "Your words stay here.",
            "A SMALL INVITATION",
            "Not every day needs to be extraordinary to be worth remembering.",
        ] {
            assert!(
                test.find(|_, element| {
                    let label = Label::try_downcast(element)
                        .is_some_and(|label| label.text.as_ref() == removed);
                    let paragraph = Paragraph::try_downcast(element)
                        .is_some_and(|p| p.spans.iter().any(|s| s.text == removed));
                    (label || paragraph).then_some(())
                })
                .is_none(),
                "decorative copy is still rendered: {removed}"
            );
        }
    }
}

#[test]
#[ignore = "Generate a documentation screenshot using fictional test entries"]
fn render_documentation_screenshot() {
    let previews = tempfile::tempdir().unwrap().keep();
    std::fs::create_dir_all("docs/images").unwrap();
    for (width, height) in [(1360., 920.), (1060., 760.)] {
        for id in theme::ThemeId::ALL {
            let (_dir, mut test) = fixture(true, width, height);
            test.animation_clock().disable();
            if id != theme::ThemeId::Sage {
                click_label(&mut test, "Theme: Sage");
                test.poll_n(std::time::Duration::from_millis(5), 30);
                click_label(&mut test, id.name());
                test.poll_n(std::time::Duration::from_millis(5), 30);
            }
            test.render_to_file(previews.join(format!("{}-{width}.png", id.name())));
            if id == theme::ThemeId::Sage && width == 1360. {
                test.render_to_file("docs/images/daymark.png");
            }
            click_label(&mut test, "Export day .md");
            test.render_to_file(previews.join(format!("{}-{width}-export.png", id.name())));
        }
    }
    eprintln!("Fictional theme previews: {}", previews.display());
}
