use freya::prelude::*;

use super::{
    components::{Action, Icon, caption, divider, icon, prose},
    editor::single_line_key_policy,
    entry_date,
    theme::COLORS,
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
        let state = journal.read();
        let entries = &state.entries;
        let searching = !state.query.trim().is_empty();
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
                timeline_row(entry, state.draft.id == Some(entry.id), journal).into_element(),
            );
        }
        rect().width(Size::px(280.)).height(Size::fill())
            .background(COLORS.surface_tertiary).padding((28., 18., 18., 18.)).spacing(24.).content(Content::flex())
            .child(rect().horizontal().spacing(11.).cross_align(Alignment::center())
                .child(rect().width(Size::px(40.)).height(Size::px(42.)).center().background(COLORS.tertiary).corner_radius(12.)
                    .child(icon(Icon::Leaf, 23., COLORS.primary)))
                .child(rect().spacing(1.)
                    .child(label().text("Daymark").font_family("DM Serif Display").font_size(28.))
                    .child(caption("A LITTLE SPACE FOR YOUR DAYS"))))
            .child(Action { text: "New entry", icon: Icon::Plus, primary: true,
                on_press: (move |()| journal.write().new_entry()).into() })
            .child(Input::new(query).on_pre_key_down(single_line_key_policy).width(Size::fill()).a11y_id(self.search_id)
                .placeholder("Search your journal…")
                .leading(icon(Icon::Search, 15., COLORS.text_secondary).into_element())
                .background(COLORS.background).placeholder_color(COLORS.text_placeholder)
                .corner_radius(8.).padding((10., 9.))
                .on_validate(move |value: InputValidator| journal.write().search(value.text().to_string())))
            .child(rect().height(Size::flex(1.)).width(Size::fill()).spacing(6.).content(Content::flex())
                .child(rect().horizontal().width(Size::fill()).padding((0., 10.)).main_align(Alignment::space_between())
                    .child(caption(if searching { "SEARCH RESULTS" } else { "TIMELINE" }))
                    .child(caption(entries.len().to_string())))
                .child(ScrollView::new().width(Size::fill()).height(Size::flex(1.)).spacing(5.)
                    .children(rows)
                    .maybe(entries.is_empty(), |el| el.child(rect().padding((24., 10.)).spacing(10.)
                        .child(prose(if searching { "No entries found." } else { "Your story starts here." }, 15., COLORS.text_secondary))
                        .child(prose(if searching { "Try another word or clear your search. Your open entry is still here." } else { "Write your first entry. It will find a home in this timeline." }, 12., COLORS.text_placeholder)))))
            )
            .child(divider())
            .child(Button::new().flat().width(Size::fill()).padding((12., 10.)).corner_radius(9.)
                .on_press(move |_| { let mut state = journal.write(); state.privacy_open = !state.privacy_open; })
                .child(rect().width(Size::fill()).spacing(8.)
                    .child(rect().horizontal().cross_align(Alignment::center()).spacing(8.)
                        .child(icon(Icon::Lock, 14., COLORS.primary))
                        .child(label().text("Just you. Just this device.").font_size(12.).color(COLORS.primary)))
                    .child(prose("No accounts. No cloud. A quiet corner that belongs to you.", 12., COLORS.text_secondary))))
            .child(rect().horizontal().width(Size::fill()).main_align(Alignment::space_between()).padding((0., 10.))
                .child(caption("DAYMARK / LOCAL JOURNAL"))
                .child(caption("v0.1")))
    }
}

fn timeline_row(
    entry: &EntrySummary,
    selected: bool,
    mut journal: State<Journal>,
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
            COLORS.tertiary
        } else {
            COLORS.surface_tertiary
        })
        .hover_background(COLORS.active)
        .border_fill(if selected {
            COLORS.border
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
                                    COLORS.primary
                                } else {
                                    COLORS.text_secondary
                                }),
                        )
                        .child(
                            paragraph()
                                .width(Size::flex(1.))
                                .max_lines(1)
                                .font_size(13.)
                                .color(COLORS.text_primary)
                                .span(title.to_string()),
                        )
                        .maybe(selected, |el| {
                            el.child(
                                rect()
                                    .width(Size::px(5.))
                                    .height(Size::px(5.))
                                    .corner_radius(99.)
                                    .background(COLORS.primary),
                            )
                        }),
                )
                .child(
                    paragraph()
                        .width(Size::fill())
                        .max_lines(1)
                        .font_size(12.)
                        .color(COLORS.text_secondary)
                        .span(if entry.preview.trim().is_empty() {
                            "A new page in your story".to_string()
                        } else {
                            entry.preview.replace('\n', " ")
                        }),
                ),
        )
}
