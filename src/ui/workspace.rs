use std::sync::atomic::Ordering;

use chrono::{Local, NaiveDate};
use freya::prelude::*;

use super::{
    components::{Action, Icon, caption, divider, icon, prose},
    editor::{WritingArea, single_line_key_policy},
    entry_date,
};
use crate::model::Journal;

#[derive(PartialEq)]
pub(super) struct Workspace {
    pub journal: State<Journal>,
}

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
        let colors = use_theme().read().colors.clone();
        let mut prompt_index = use_state(|| 0usize);
        let export_open = use_state(|| false);
        let state = journal.read();
        let date = entry_date(&state.draft.created_at);
        let today = date == Local::now().date_naive();
        let words = state.draft.body.split_whitespace().count();
        let unsaved = state.unsaved.load(Ordering::SeqCst);
        let generation = state.editor_generation;
        rect()
            .width(Size::flex(1.))
            .height(Size::fill())
            .content(Content::flex())
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(78.))
                    .horizontal()
                    .padding((0., 32.))
                    .cross_align(Alignment::center())
                    .main_align(Alignment::space_between())
                    .child(
                        rect()
                            .horizontal()
                            .spacing(12.)
                            .cross_align(Alignment::center())
                            .child(icon(Icon::Leaf, 16., colors.primary))
                            .child(label().text("Journal").font_size(13.)),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .spacing(10.)
                            .cross_align(Alignment::center())
                            .child(icon(Icon::Lock, 13., colors.primary))
                            .child(caption("LOCAL ONLY"))
                            .child(Action {
                                text: if state.focus_mode {
                                    "Show timeline"
                                } else {
                                    "Focus"
                                },
                                icon: Icon::Focus,
                                primary: false,
                                on_press: (move |()| {
                                    let mut state = journal.write();
                                    state.focus_mode = !state.focus_mode;
                                })
                                .into(),
                            }),
                    ),
            )
            .child(divider())
            .child(
                rect()
                    .height(Size::flex(1.))
                    .width(Size::fill())
                    .content(Content::flex())
                    .padding((32., 44., 24., 44.))
                    .cross_align(Alignment::center())
                    .spacing(22.)
                    .child(DayExport {
                        journal,
                        date,
                        open: export_open,
                    })
                    .maybe_child(state.error.as_ref().map(|error| {
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .padding(14.)
                            .spacing(10.)
                            .background(colors.surface_tertiary)
                            .corner_radius(8.)
                            .child(prose(error.clone(), 13., colors.error))
                            .child(Action {
                                text: "Retry save",
                                icon: Icon::Arrow,
                                primary: false,
                                on_press: (move |()| {
                                    journal.write().save();
                                })
                                .into(),
                            })
                    }))
                    .maybe(state.privacy_open, |el| {
                        el.child(privacy_panel(journal, &colors))
                    })
                    .child(
                        rect()
                            .height(Size::flex(1.))
                            .width(Size::fill())
                            .content(Content::flex())
                            .max_width(Size::px(960.))
                            .horizontal()
                            .spacing(24.)
                            .child(
                                rect()
                                    .width(Size::px(58.))
                                    .spacing(7.)
                                    .cross_align(Alignment::center())
                                    .padding((15., 0.))
                                    .child(
                                        label()
                                            .text(date.format("%d").to_string())
                                            .font_family("DM Serif Display")
                                            .font_size(44.)
                                            .color(colors.primary),
                                    )
                                    .child(caption(if today {
                                        "TODAY".to_string()
                                    } else {
                                        date.format("%a").to_string().to_uppercase()
                                    }))
                                    .child(caption(date.format("%b %Y").to_string())),
                            )
                            .child(
                                rect()
                                    .width(Size::flex(1.))
                                    .height(Size::fill())
                                    .content(Content::flex())
                                    .padding((26., 30., 18., 30.))
                                    .spacing(18.)
                                    .background(colors.surface_secondary)
                                    .corner_radius(14.)
                                    .border(Border::new().width(1.).fill(colors.border))
                                    .child(
                                        rect()
                                            .width(Size::fill())
                                            .horizontal()
                                            .main_align(Alignment::space_between())
                                            .cross_align(Alignment::center())
                                            .child(caption(
                                                date.format("%A, %B %-d")
                                                    .to_string()
                                                    .to_uppercase(),
                                            ))
                                            .maybe(state.draft.id.is_some(), |el| {
                                                el.child(
                                                    Button::new()
                                                        .flat()
                                                        .compact()
                                                        .padding(6.)
                                                        .enabled(!unsaved)
                                                        .on_press(move |_| {
                                                            journal.write().delete_pending = true
                                                        })
                                                        .child(
                                                            rect()
                                                                .horizontal()
                                                                .spacing(6.)
                                                                .cross_align(Alignment::center())
                                                                .child(icon(
                                                                    Icon::Trash,
                                                                    14.,
                                                                    colors.text_secondary,
                                                                ))
                                                                .child(
                                                                    label()
                                                                        .text("Delete")
                                                                        .font_size(12.)
                                                                        .color(
                                                                            colors.text_secondary,
                                                                        ),
                                                                ),
                                                        ),
                                                )
                                            }),
                                    )
                                    .maybe(state.delete_pending, |el| {
                                        el.child(delete_confirmation(journal, &colors))
                                    })
                                    .child(
                                        WritingArea {
                                            journal,
                                            key: DiffKey::None,
                                        }
                                        .key(generation),
                                    )
                                    .child(divider())
                                    .child(
                                        rect()
                                            .width(Size::fill())
                                            .horizontal()
                                            .cross_align(Alignment::center())
                                            .main_align(Alignment::space_between())
                                            .child(
                                                rect()
                                                    .horizontal()
                                                    .spacing(7.)
                                                    .cross_align(Alignment::center())
                                                    .child(icon(
                                                        if unsaved {
                                                            Icon::Close
                                                        } else {
                                                            Icon::Check
                                                        },
                                                        14.,
                                                        if unsaved {
                                                            colors.error
                                                        } else {
                                                            colors.primary
                                                        },
                                                    ))
                                                    .child(caption(if unsaved {
                                                        "Not saved · your draft is kept open"
                                                    } else if state.draft.id.is_some() {
                                                        "Saved to this device"
                                                    } else {
                                                        "New entry · saves automatically"
                                                    })),
                                            )
                                            .child(caption(format!(
                                                "{words} {} · ~{} min read",
                                                if words == 1 { "word" } else { "words" },
                                                reading_minutes(words),
                                            ))),
                                    ),
                            ),
                    )
                    .maybe(!*export_open.read(), |el| {
                        el.child(
                            rect()
                                .width(Size::fill())
                                .max_width(Size::px(960.))
                                .padding((0., 0., 0., 82.))
                                .child(prompt(
                                    *prompt_index.read(),
                                    (move |()| {
                                        let next = next_prompt(*prompt_index.peek());
                                        *prompt_index.write() = next;
                                    })
                                    .into(),
                                    &colors,
                                )),
                        )
                    })
                    .child(
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .horizontal()
                            .main_align(Alignment::end())
                            .child(caption(if cfg!(target_os = "macos") {
                                "⌘ N  new entry  ·  ⌘ F  search"
                            } else {
                                "Ctrl N  new entry  ·  Ctrl F  search"
                            })),
                    ),
            )
    }
}

fn delete_confirmation(mut journal: State<Journal>, colors: &ColorsSheet) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .padding(12.)
        .spacing(10.)
        .background(colors.tertiary)
        .corner_radius(8.)
        .child(prose(
            "Delete this entry permanently? This cannot be undone.",
            13.,
            colors.text_primary,
        ))
        .child(
            rect()
                .horizontal()
                .spacing(10.)
                .child(Action {
                    text: "Keep entry",
                    icon: Icon::Close,
                    primary: false,
                    on_press: (move |()| journal.write().delete_pending = false).into(),
                })
                .child(Action {
                    text: "Delete permanently",
                    icon: Icon::Trash,
                    primary: false,
                    on_press: (move |()| journal.write().confirm_delete()).into(),
                }),
        )
}

fn privacy_panel(mut journal: State<Journal>, colors: &ColorsSheet) -> impl IntoElement {
    let path = journal.read().store.path.display().to_string();
    rect().width(Size::fill()).max_width(Size::px(960.)).padding(18.).spacing(8.)
        .background(colors.tertiary).corner_radius(10.)
        .child(label().text("Privacy and storage").font_size(16.).color(colors.primary))
        .child(prose("Daymark has no account, telemetry, sync, or network requests. The SQLite database is local, but NOT encrypted. Use OS disk encryption for protection at rest. Close Daymark before copying the database for a backup.", 13., colors.text_primary))
        .child(prose(path, 12., colors.text_secondary))
        .child(Action { text: "Close", icon: Icon::Close, primary: false,
            on_press: (move |()| journal.write().privacy_open = false).into() })
}

#[derive(PartialEq)]
struct DayExport {
    journal: State<Journal>,
    date: NaiveDate,
    open: State<bool>,
}

impl Component for DayExport {
    fn render_key(&self) -> DiffKey {
        DiffKey::from(&self.date)
    }

    fn render(&self) -> impl IntoElement {
        let journal = self.journal;
        let colors = use_theme().read().colors.clone();
        let date = self.date;
        let mut open = self.open;
        let filename = format!("{date}.md");
        let path = use_state(|| {
            directories::UserDirs::new()
                .map(|dirs| dirs.home_dir().join(&filename))
                .unwrap_or_else(|| std::path::PathBuf::from(&filename))
                .to_string_lossy()
                .into_owned()
        });
        let mut result = use_state(|| None::<Result<(), String>>);
        rect()
            .width(Size::fill())
            .max_width(Size::px(960.))
            .spacing(10.)
            .child(
                rect()
                    .width(Size::fill())
                    .horizontal()
                    .main_align(Alignment::space_between())
                    .cross_align(Alignment::center())
                    .child(caption("Markdown exports are plaintext, not encrypted."))
                    .child(Action {
                        text: "Export day .md",
                        icon: Icon::Arrow,
                        primary: false,
                        on_press: (move |()| {
                            *result.write() = None;
                            *open.write() = true;
                        })
                        .into(),
                    }),
            )
            .maybe(*open.read(), |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .padding(16.)
                        .background(colors.surface_tertiary)
                        .corner_radius(10.)
                        .on_global_key_down(move |event: Event<KeyboardEventData>| {
                            if event.key == Key::Named(NamedKey::Escape) {
                                *open.write() = false;
                            }
                        })
                        .child(
                            rect()
                                .width(Size::fill())
                                .spacing(12.)
                                .a11y_alt("Markdown export destination")
                                .child(label().text(format!("Export {date}")).font_size(18.))
                                .child(prose(
                                    "Export all entries on this day, including the open draft. This does not save the journal. This file is plaintext, not encrypted. Choose an existing local folder; existing files and symbolic links are refused.",
                                    12.,
                                    colors.text_secondary,
                                ))
                                .child(
                                    Input::new(path)
                                        .on_pre_key_down(single_line_key_policy)
                                        .auto_focus(true)
                                        .width(Size::fill())
                                        .placeholder("Local path for Markdown export")
                                        .leading(label().text("Export path").font_size(12.)),
                                )
                                .child(
                                    rect()
                                        .horizontal()
                                        .spacing(10.)
                                        .child(Action {
                                            text: "Write .md file",
                                            icon: Icon::Arrow,
                                            primary: false,
                                            on_press: (move |()| {
                                                let exported = journal
                                                    .read()
                                                    .export_day(date, std::path::Path::new(path.peek().as_str()))
                                                    .map_err(|err| format!("Export failed. {err:#}"));
                                                if exported.is_ok() {
                                                    *open.write() = false;
                                                }
                                                *result.write() = Some(exported);
                                            })
                                            .into(),
                                        })
                                        .child(Action {
                                            text: "Cancel export",
                                            icon: Icon::Close,
                                            primary: false,
                                            on_press: (move |()| *open.write() = false).into(),
                                        }),
                                )
                                .maybe_child(result.read().as_ref().map(|result| export_feedback(result, &colors))),
                        ),
                )
            })
            .maybe(!*open.read(), |el| {
                el.maybe_child(result.read().as_ref().map(|result| export_feedback(result, &colors)))
            })
    }
}

fn export_feedback(result: &Result<(), String>, colors: &ColorsSheet) -> impl IntoElement {
    let (message, color) = match result {
        Ok(()) => (
            "Export complete. Plaintext Markdown written; journal save status is unchanged.",
            colors.primary,
        ),
        Err(error) => (error.as_str(), colors.error),
    };
    prose(message.to_string(), 12., color)
}

const PROMPTS: [&str; 4] = [
    "What happened today?",
    "What went well, and what was difficult?",
    "What did you learn today?",
    "What do you want to do tomorrow?",
];

fn next_prompt(index: usize) -> usize {
    (index + 1) % PROMPTS.len()
}

fn reading_minutes(words: usize) -> usize {
    words.div_ceil(200)
}

fn prompt(index: usize, on_next: EventHandler<()>, colors: &ColorsSheet) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .padding((14., 22.))
        .spacing(7.)
        .background(colors.surface_tertiary)
        .corner_radius(10.)
        .child(
            rect()
                .width(Size::fill())
                .horizontal()
                .cross_align(Alignment::center())
                .main_align(Alignment::space_between())
                .child(
                    label()
                        .text("WRITING PROMPT")
                        .font_size(10.)
                        .color(colors.primary)
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    Button::new()
                        .flat()
                        .compact()
                        .on_press(move |_| on_next.call(()))
                        .child(label().text("Next prompt").font_size(12.)),
                ),
        )
        .child(prose(PROMPTS[index], 14., colors.text_secondary))
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::HashMap,
        sync::{Arc, atomic::AtomicBool},
    };

    use freya_testing::{TestingNode, prelude::*};

    use super::*;
    use crate::{storage::Store, ui::DaymarkApp};

    fn fixture(
        width: f32,
        height: f32,
        failed: bool,
    ) -> (tempfile::TempDir, TestingRunner, Arc<AtomicBool>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("data")).unwrap();
        let unsaved = Arc::new(AtomicBool::new(false));
        let mut journal = Journal::new(store, unsaved.clone()).unwrap();
        journal.draft.created_at = "2026-10-24T08:00:00+00:00".into();
        journal.edit_body("One fictional thought".into());
        if failed {
            let mut external = journal.draft.clone();
            external.body = "Changed in another window".into();
            journal.store.save(&mut external).unwrap();
            journal.edit_body("Failed draft kept here".into());
        }
        let app = DaymarkApp {
            initial: RefCell::new(Some(journal)),
        };
        let (mut test, _) = TestingRunner::new(
            move || {
                let window = use_hook(|| Platform::get().current_window());
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    // TestingRunner omits the native Tab integration wrapper.
                    .on_global_key_down(move |event: Event<KeyboardEventData>| {
                        if event.key == Key::Named(NamedKey::Tab) {
                            let movement = AccessibilityFocusMovement::OutsideGroup;
                            let strategy = if event.modifiers.contains(Modifiers::SHIFT) {
                                AccessibilityFocusStrategy::Backward(movement)
                            } else {
                                AccessibilityFocusStrategy::Forward(movement)
                            };
                            window.send(UserEvent::FocusAccessibilityNode(strategy));
                        }
                    })
                    .child(app.render())
                    .into_element()
            },
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
        test.animation_clock().disable();
        (dir, test, unsaved)
    }

    fn text_node(test: &TestingRunner, text: &str) -> TestingNode {
        test.find_many(|node, element| {
            let matches = Label::try_downcast(element)
                .is_some_and(|label| label.text.as_ref() == text)
                || Paragraph::try_downcast(element)
                    .is_some_and(|paragraph| paragraph.spans.iter().any(|span| span.text == text));
            matches.then_some(node)
        })
        .pop()
        .unwrap_or_else(|| panic!("missing fictional UI text: {text}"))
    }

    fn click_text(test: &mut TestingRunner, text: &str) {
        let point = text_node(test, text).layout().area.center();
        test.click_cursor((point.x as f64, point.y as f64));
        test.sync_and_update();
        test.render();
    }

    fn choose_export_path(test: &mut TestingRunner, path: &std::path::Path) {
        click_text(test, "Export day .md");
        let default = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .join("2026-10-24.md");
        click_text(test, default.to_str().unwrap());
        test.send_event(PlatformEvent::Keyboard {
            name: KeyboardEventName::KeyDown,
            key: Key::Character("a".into()),
            code: Code::Unidentified,
            modifiers: Modifiers::ctrl_or_meta(),
        });
        test.sync_and_update();
        test.write_text(path.to_str().unwrap());
    }

    #[test]
    fn prompt_and_export_controls_preserve_entries_at_supported_sizes() {
        for (width, height) in [(1360., 920.), (1060., 760.)] {
            let (dir, mut test, unsaved) = fixture(width, height, false);
            let store = Store::open(&dir.path().join("data")).unwrap();
            let entry = store.get(store.list("").unwrap()[0].id).unwrap();
            click_text(&mut test, "Next prompt");
            assert!(text_node(&test, PROMPTS[1]).layout().area.height() > 0.);
            assert_eq!(store.get(entry.id.unwrap()).unwrap(), entry);
            assert!(
                text_node(&test, "3 words · ~1 min read")
                    .layout()
                    .area
                    .height()
                    > 0.
            );
            click_text(&mut test, "One fictional thought");
            test.press_key(Key::Named(NamedKey::End));
            test.write_text(" again");
            assert!(
                text_node(&test, "4 words · ~1 min read")
                    .layout()
                    .area
                    .height()
                    > 0.
            );
            let entry = store.get(entry.id.unwrap()).unwrap();
            let path = dir.path().join("2026-10-24.md");
            choose_export_path(&mut test, &path);
            let area = text_node(&test, "Write .md file").layout().area;
            assert!(area.min_y() >= 0. && area.max_y() <= height);
            assert!(area.min_x() >= 0. && area.max_x() <= width);
            click_text(&mut test, "Write .md file");
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                "# 2026-10-24\n\n## \n\nOne fictional thought again\n\n"
            );
            assert!(
                text_node(
                    &test,
                    "Export complete. Plaintext Markdown written; journal save status is unchanged."
                )
                .layout()
                .area
                .height()
                    > 0.
            );
            assert!(
                text_node(&test, "Saved to this device")
                    .layout()
                    .area
                    .height()
                    > 0.
            );
            click_text(&mut test, "Export day .md");
            click_text(&mut test, "Write .md file");
            assert!(
                test.find(|_, element| Paragraph::try_downcast(element)
                    .filter(|paragraph| paragraph
                        .spans
                        .iter()
                        .any(|span| span.text.starts_with("Export failed.")))
                    .map(|_| ()))
                    .is_some()
            );
            assert!(
                text_node(&test, "Saved to this device")
                    .layout()
                    .area
                    .height()
                    > 0.
            );
            assert!(!unsaved.load(Ordering::SeqCst));
            assert_eq!(store.get(entry.id.unwrap()).unwrap(), entry);
        }
    }

    #[test]
    fn exporting_failed_draft_keeps_the_journal_failure_visible_and_close_blocked() {
        let (dir, mut test, unsaved) = fixture(1060., 760., true);
        assert!(unsaved.load(Ordering::SeqCst));
        let path = dir.path().join("rescue.md");
        choose_export_path(&mut test, &path);
        click_text(&mut test, "Write .md file");
        assert!(
            std::fs::read_to_string(path)
                .unwrap()
                .contains("Failed draft kept here")
        );
        assert!(unsaved.load(Ordering::SeqCst));
        assert!(text_node(&test, "Retry save").layout().area.height() > 0.);
        assert!(
            text_node(&test, "Not saved · your draft is kept open")
                .layout()
                .area
                .height()
                > 0.
        );
        assert!(
            text_node(
                &test,
                "Export complete. Plaintext Markdown written; journal save status is unchanged."
            )
            .layout()
            .area
            .height()
                > 0.
        );
    }

    #[test]
    fn inline_export_supports_keyboard_cancel_and_escape_without_hiding_the_editor() {
        for close_with_escape in [false, true] {
            let (dir, mut test, unsaved) = fixture(1060., 760., false);
            let store = Store::open(&dir.path().join("data")).unwrap();
            let entry = store.get(store.list("").unwrap()[0].id).unwrap();
            let path = dir.path().join("cancelled.md");
            choose_export_path(&mut test, &path);
            let editor = text_node(&test, "One fictional thought").layout().area;
            assert!(
                editor.height() > 0. && editor.min_y() + 32. <= 760.,
                "{editor:?}"
            );
            if close_with_escape {
                test.press_key(Key::Named(NamedKey::Escape));
            } else {
                test.press_key(Key::Named(NamedKey::Tab));
                test.sync_and_update();
                test.press_key(Key::Named(NamedKey::Tab));
                test.sync_and_update();
                test.press_key(Key::Named(NamedKey::Enter));
            }
            test.sync_and_update();
            assert!(
                test.find(|_, element| Label::try_downcast(element)
                    .filter(|label| label.text.as_ref() == "Cancel export")
                    .map(|_| ()))
                    .is_none()
            );
            assert_eq!(store.get(entry.id.unwrap()).unwrap(), entry);
            assert!(!unsaved.load(Ordering::SeqCst));
            assert!(!path.exists());
        }
    }

    #[test]
    fn reading_estimate_rounds_up_at_two_hundred_words_per_minute() {
        for (words, minutes) in [(0, 0), (1, 1), (199, 1), (200, 1), (201, 2), (400, 2)] {
            assert_eq!(reading_minutes(words), minutes);
        }
    }

    #[test]
    fn offline_prompts_cycle_in_order_and_wrap() {
        let mut index = 0;
        for expected in 1..=PROMPTS.len() {
            index = next_prompt(index);
            assert_eq!(index, expected % PROMPTS.len());
            assert!(!PROMPTS[index].is_empty());
        }
        assert_eq!(index, 0);
    }
}
