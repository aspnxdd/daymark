use chrono::{Datelike, Local, NaiveDate};
use freya::prelude::*;

use super::{
    components::{Action, Icon, caption, divider, icon, prose},
    editor::single_line_key_policy,
    entry_date,
    theme::ThemeSelector,
};
use crate::{model::Journal, storage::EntrySummary};

#[derive(PartialEq)]
pub(super) struct Sidebar {
    pub journal: State<Journal>,
    pub search_id: AccessibilityId,
}

impl Component for Sidebar {
    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
        let colors = use_theme().read().colors.clone();
        let state = journal.read();
        let entries = &state.entries;
        let searching = !state.query.trim().is_empty();
        let day_selected = state.selected_day.is_some();
        let query = journal.into_writable().map(|s| &s.query, |s| &mut s.query);
        let mut rows: Vec<Element> = Vec::new();
        let mut previous_month = String::new();
        for entry in entries {
            let date = entry_date(&entry.created_at);
            let month = date.format("%B %Y").to_string();
            if month != previous_month {
                rows.push(
                    rect()
                        .padding((18., 10., 8., 10.))
                        .child(caption(month.to_uppercase()))
                        .into(),
                );
                previous_month = month;
            }
            rows.push(
                timeline_row(entry, state.draft.id == Some(entry.id), journal, &colors)
                    .into_element(),
            );
        }
        rect().width(Size::px(280.)).height(Size::fill())
            .background(colors.surface_tertiary).padding((20., 18., 14., 18.)).spacing(12.).content(Content::flex())
            .child(rect().horizontal().spacing(11.).cross_align(Alignment::center())
                .child(rect().width(Size::px(40.)).height(Size::px(42.)).center().background(colors.tertiary).corner_radius(12.)
                    .child(icon(Icon::Leaf, 23., colors.primary)))
                .child(label().text("Daymark").font_family("DM Serif Display").font_size(28.)))
            .child(Action { text: "New entry", icon: Icon::Plus, primary: true,
                on_press: (move |()| journal.write().new_entry()).into() })
            .child(Input::new(query).on_pre_key_down(single_line_key_policy).width(Size::fill()).a11y_id(self.search_id)
                .placeholder("Search your journal…")
                .leading(icon(Icon::Search, 15., colors.text_secondary).into_element())
                .background(colors.background).placeholder_color(colors.text_placeholder)
                .corner_radius(8.).padding((10., 9.))
                .on_validate(move |value: InputValidator| journal.write().search(value.text().to_string())))
            .child(MonthCalendar { journal })
            .child(rect().height(Size::flex(1.)).width(Size::fill()).spacing(6.).content(Content::flex())
                .child(rect().horizontal().width(Size::fill()).padding((0., 10.)).main_align(Alignment::space_between())
                    .child(caption(if searching { "SEARCH RESULTS".to_string() } else if let Some(day) = state.selected_day { day.format("%b %-d, %Y").to_string().to_uppercase() } else { "TIMELINE".to_string() }))
                    .child(caption(entries.len().to_string())))
                .child(ScrollView::new().width(Size::fill()).height(Size::flex(1.)).spacing(5.)
                    .children(rows)
                    .maybe(entries.is_empty(), |el| el.child(rect().padding((24., 10.)).spacing(10.)
                        .child(prose(if searching { "No entries found." } else if day_selected { "No entries on this day." } else { "No entries yet." }, 15., colors.text_secondary))
                        .child(prose(if searching { "Try another word or choose All entries. Your open entry is still here." } else if day_selected { "Choose another date or All entries. Your open entry is still here." } else { "Select New entry to get started." }, 12., colors.text_placeholder)))))
            )
            .child(divider())
            .child(Button::new().flat().width(Size::fill()).padding((12., 10.)).corner_radius(9.)
                .on_press(move |_| { let mut state = journal.write(); state.privacy_open = !state.privacy_open; })
                .child(rect().horizontal().width(Size::fill()).cross_align(Alignment::center()).spacing(8.)
                    .child(icon(Icon::Lock, 14., colors.primary))
                    .child(label().text("Privacy and storage").font_size(12.).color(colors.primary))))
            .child(ThemeSelector)
    }
}

#[derive(PartialEq)]
struct MonthCalendar {
    journal: State<Journal>,
}

impl Component for MonthCalendar {
    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
        let state = journal.read();
        let colors = use_theme().read().colors.clone();
        let month = CalendarMonth::new(
            state.calendar_month.into(),
            state.selected_day.map(Into::into),
            WeekStart::Monday,
        )
        .expect("Journal keeps a valid calendar month");
        let counts = state.entry_days.clone();
        let today = Local::now().date_naive();
        let previous = calendar_date(month.previous_month);
        let next = calendar_date(month.next_month);
        let first = calendar_date(month.first_day);
        rect()
            .width(Size::fill())
            .spacing(5.)
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .cross_align(Alignment::center())
                    .main_align(Alignment::space_between())
                    .child(
                        Button::new()
                            .flat()
                            .width(Size::px(28.))
                            .height(Size::px(28.))
                            .padding(0.)
                            .enabled(previous != first)
                            .on_press(move |_| journal.write().view_month(previous))
                            .child(calendar_label("Previous month", "‹")),
                    )
                    .child(
                        label()
                            .text(first.format("%B %Y").to_string())
                            .font_size(13.),
                    )
                    .child(
                        Button::new()
                            .flat()
                            .width(Size::px(28.))
                            .height(Size::px(28.))
                            .padding(0.)
                            .enabled(next != first)
                            .on_press(move |_| journal.write().view_month(next))
                            .child(calendar_label("Next month", "›")),
                    ),
            )
            .child(
                rect().horizontal().children(
                    ["M", "T", "W", "T", "F", "S", "S"]
                        .into_iter()
                        .map(|day| rect().width(Size::px(34.)).center().child(caption(day))),
                ),
            )
            .child(
                CalendarGrid::new(month, move |day| {
                    let date = calendar_date(day.date);
                    let count = counts.get(&date).copied().unwrap_or(0);
                    let is_today = date == today;
                    let name = format!(
                        "{}, {count} {}, {}{}",
                        date.format("%A, %B %-d, %Y"),
                        if count == 1 { "entry" } else { "entries" },
                        if day.selected {
                            "selected"
                        } else {
                            "not selected"
                        },
                        if is_today { ", today" } else { "" },
                    );
                    Button::new()
                        .key(date)
                        .width(Size::px(34.))
                        .height(Size::px(28.))
                        .padding(0.)
                        .corner_radius(5.)
                        .background(if day.selected {
                            colors.tertiary
                        } else {
                            colors.surface_tertiary
                        })
                        .hover_background(colors.active)
                        .border_fill(if is_today {
                            colors.primary
                        } else {
                            Color::TRANSPARENT
                        })
                        .on_press(move |_| journal.write().select_day(date))
                        .child(
                            calendar_label(name, date.day().to_string())
                                .color(if day.selected {
                                    colors.primary
                                } else if day.in_month {
                                    colors.text_primary
                                } else {
                                    colors.text_placeholder
                                })
                                .child(
                                    rect()
                                        .width(Size::px(3.))
                                        .height(Size::px(3.))
                                        .corner_radius(99.)
                                        .background(if count > 0 {
                                            colors.primary
                                        } else {
                                            Color::TRANSPARENT
                                        }),
                                ),
                        )
                        .into_element()
                })
                .width(Size::fill()),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .main_align(Alignment::space_between())
                    .child(caption("• entries · outline: today"))
                    .child(caption("MON–SUN")),
            )
            .child(
                rect()
                    .horizontal()
                    .width(Size::fill())
                    .spacing(8.)
                    .child(
                        Button::new()
                            .flat()
                            .padding((5., 8.))
                            .on_press(move |_| journal.write().select_day(today))
                            .child(label().text("Today").font_size(12.)),
                    )
                    .child(
                        Button::new()
                            .flat()
                            .padding((5., 8.))
                            .on_press(move |_| journal.write().show_all_entries())
                            .child(label().text("All entries").font_size(12.)),
                    ),
            )
    }
}

fn calendar_date(date: CalendarDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year, date.month, date.day)
        .expect("Freya calendar produces valid dates")
}

fn calendar_label(name: impl Into<String>, text: impl Into<String>) -> Rect {
    // Button takes its accessible name from an immediate child's value in pinned Freya.
    rect()
        .center()
        .spacing(1.)
        .a11y_builder(|node| node.set_value(name.into()))
        .child(label().text(text.into()).font_size(12.))
}

fn timeline_row(
    entry: &EntrySummary,
    selected: bool,
    mut journal: State<Journal>,
    colors: &ColorsSheet,
) -> impl IntoElement {
    let id = entry.id;
    let date = entry_date(&entry.created_at);
    let title = if entry.title.trim().is_empty() {
        "Untitled entry"
    } else {
        entry.title.trim()
    };
    Button::new()
        .key(id)
        .width(Size::fill())
        .padding((13., 12.))
        .corner_radius(9.)
        .background(if selected {
            colors.tertiary
        } else {
            colors.surface_tertiary
        })
        .hover_background(colors.active)
        .border_fill(if selected {
            colors.border
        } else {
            Color::TRANSPARENT
        })
        .on_press(move |_| journal.write().select(id))
        .child(
            rect()
                .width(Size::fill())
                .spacing(6.)
                .child(
                    rect()
                        .horizontal()
                        .width(Size::fill())
                        .cross_align(Alignment::center())
                        .spacing(8.)
                        .content(Content::flex())
                        .child(
                            label()
                                .text(date.format("%d").to_string())
                                .font_family("DM Serif Display")
                                .font_size(23.)
                                .color(if selected {
                                    colors.primary
                                } else {
                                    colors.text_secondary
                                }),
                        )
                        .child(
                            paragraph()
                                .width(Size::flex(1.))
                                .max_lines(1)
                                .font_size(13.)
                                .color(colors.text_primary)
                                .span(title.to_string()),
                        )
                        .maybe(selected, |el| {
                            el.child(
                                rect()
                                    .width(Size::px(5.))
                                    .height(Size::px(5.))
                                    .corner_radius(99.)
                                    .background(colors.primary),
                            )
                        }),
                )
                .child(
                    paragraph()
                        .width(Size::fill())
                        .max_lines(1)
                        .font_size(12.)
                        .color(colors.text_secondary)
                        .span(if entry.preview.trim().is_empty() {
                            "No text".to_string()
                        } else {
                            entry.preview.replace('\n', " ")
                        }),
                ),
        )
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        rc::Rc,
        sync::{Arc, atomic::AtomicBool},
    };

    use freya_testing::{TestingNode, prelude::*};

    use super::*;
    use crate::{
        storage::{Entry, Store},
        ui::theme::{ThemeId, ThemePreference},
    };

    fn fixture() -> (tempfile::TempDir, TestingRunner, State<Journal>) {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        for (date, title) in [
            ("2024-02-29T09:00:00+00:00", "Morning 100%_quiet"),
            ("2024-02-29T18:00:00+00:00", "Evening 100%_quiet"),
            ("2024-03-01T09:00:00+00:00", "Next day"),
        ] {
            let mut entry = Entry::blank();
            entry.created_at = date.into();
            entry.title = title.into();
            store.save(&mut entry).unwrap();
        }
        let mut journal = Journal::new(store, Arc::new(AtomicBool::new(false))).unwrap();
        journal.view_month(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        let initial = RefCell::new(Some(journal));
        let handle = Rc::new(Cell::new(None));
        let state = handle.clone();
        let (mut test, _) = TestingRunner::new(
            move || {
                use_init_theme(|| ThemeId::Sage.theme());
                let journal = use_state(|| initial.borrow_mut().take().unwrap());
                let preference = use_state(|| ThemePreference::load(&journal.peek().store.path));
                use_provide_context(|| preference);
                state.set(Some(journal));
                let search_id = use_a11y();
                let window = use_hook(|| Platform::get().current_window());
                // TestingRunner omits Freya's native integration wrapper that handles Tab.
                rect()
                    .on_global_key_down(move |event: Event<KeyboardEventData>| {
                        if event.key == Key::Named(NamedKey::Tab) {
                            window.send(UserEvent::FocusAccessibilityNode(
                                AccessibilityFocusStrategy::Forward(
                                    AccessibilityFocusMovement::OutsideGroup,
                                ),
                            ));
                        }
                    })
                    .child(Sidebar { journal, search_id })
                    .into_element()
            },
            Size2D::new(280., 760.),
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
        (dir, test, handle.get().unwrap())
    }

    fn named(test: &TestingRunner, name: &str) -> TestingNode {
        test.find(|node, element| {
            (element.accessibility().builder.value() == Some(name)).then_some(node)
        })
        .unwrap_or_else(|| panic!("calendar control not found: {name}"))
    }

    fn click(test: &mut TestingRunner, node: TestingNode) {
        let area = node.layout().area;
        assert!(area.min_x() >= 0. && area.max_x() <= 280.);
        assert!(area.min_y() >= 0. && area.max_y() <= 760.);
        let point = area.center();
        test.click_cursor((point.x as f64, point.y as f64));
        test.sync_and_update();
        test.render();
    }

    fn click_named(test: &mut TestingRunner, name: &str) {
        let node = named(test, name);
        click(test, node);
    }

    fn click_text(test: &mut TestingRunner, text: &str) {
        let node = test
            .find(|node, element| {
                Label::try_downcast(element)
                    .filter(|label| label.text.as_ref() == text)
                    .map(|_| node)
            })
            .unwrap();
        click(test, node);
    }

    #[test]
    fn calendar_month_boundaries_and_leap_days() {
        for (year, month, day_count) in [(2024, 2, 29), (2025, 2, 28), (2024, 12, 31)] {
            let first = CalendarDate::new(year, month, 1);
            let grid = CalendarMonth::new(first, None, WeekStart::Monday).unwrap();
            let days: Vec<_> = grid.days.iter().filter(|day| day.in_month).collect();
            assert_eq!(days.len(), day_count);
            assert_eq!(days[0].date.day, 1);
            assert_eq!(days.last().unwrap().date.day, day_count as u32);
            assert_eq!(grid.days.len() % 7, 0);
            assert_eq!(
                calendar_date(grid.days[0].date).weekday(),
                chrono::Weekday::Mon
            );
        }
        let december =
            CalendarMonth::new(CalendarDate::new(2024, 12, 1), None, WeekStart::Monday).unwrap();
        assert_eq!(december.next_month, CalendarDate::new(2025, 1, 1));
        let january = CalendarMonth::new(december.next_month, None, WeekStart::Monday).unwrap();
        assert_eq!(january.previous_month, december.first_day);
    }

    #[test]
    fn calendar_day_filters_search_and_keeps_every_entry_accessible() {
        let (_dir, mut test, journal) = fixture();
        let generation = journal.read().editor_generation;
        let draft = journal.read().draft.clone();
        click_named(
            &mut test,
            "Thursday, February 29, 2024, 2 entries, not selected",
        );
        assert_eq!(journal.read().entries.len(), 2);
        for title in ["Morning 100%_quiet", "Evening 100%_quiet"] {
            let row = test
                .find(|node, element| {
                    Paragraph::try_downcast(element)
                        .filter(|p| p.spans.iter().any(|span| span.text == title))
                        .map(|_| node)
                })
                .unwrap();
            assert!(row.layout().area.height() > 0.);
        }
        let search = test
            .find(|node, element| {
                Paragraph::try_downcast(element)
                    .filter(|p| {
                        p.spans
                            .iter()
                            .any(|span| span.text == "Search your journal…")
                    })
                    .map(|_| node)
            })
            .unwrap();
        click(&mut test, search);
        test.write_text("Morning 100%_");
        assert_eq!(journal.read().entries.len(), 1);
        assert_eq!(
            journal.read().entry_days[&NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()],
            2
        );
        click_named(
            &mut test,
            "Wednesday, February 28, 2024, 0 entries, not selected",
        );
        assert!(journal.read().entries.is_empty());
        assert_eq!(journal.read().draft, draft);
        assert_eq!(journal.read().editor_generation, generation);
        click_text(&mut test, "All entries");
        assert!(journal.read().query.is_empty());
        assert!(journal.read().selected_day.is_none());
        assert_eq!(journal.read().entries.len(), 3);
        click_text(&mut test, "Today");
        assert_eq!(journal.read().selected_day, Some(Local::now().date_naive()));
    }

    #[test]
    fn month_controls_and_day_selection_work_with_keyboard() {
        let (_dir, mut test, mut journal) = fixture();
        journal
            .write()
            .view_month(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        test.sync_and_update();
        click_named(&mut test, "Next month");
        assert_eq!(
            journal.read().calendar_month,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        );
        test.press_key(Key::Named(NamedKey::Enter));
        assert_eq!(
            journal.read().calendar_month,
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()
        );
        test.press_key(Key::Named(NamedKey::Tab));
        test.sync_and_update();
        test.press_key(Key::Named(NamedKey::Enter));
        assert_eq!(
            journal.read().selected_day,
            Some(NaiveDate::from_ymd_opt(2025, 1, 27).unwrap())
        );
    }
}
