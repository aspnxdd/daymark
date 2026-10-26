use std::sync::atomic::Ordering;

use chrono::Local;
use freya::prelude::*;

use super::{
    components::{Action, Icon, caption, divider, icon, prose},
    editor::WritingArea,
    entry_date,
    theme::COLORS,
};
use crate::model::Journal;

#[derive(PartialEq)]
pub(super) struct Workspace {
    pub journal: State<Journal>,
}

impl Component for Workspace {
    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
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
                            .child(icon(Icon::Leaf, 16., COLORS.primary))
                            .child(label().text("Your journal").font_size(13.))
                            .child(caption("/"))
                            .child(caption(if state.draft.id.is_none() {
                                "A fresh page"
                            } else {
                                "A moment kept"
                            })),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .spacing(10.)
                            .cross_align(Alignment::center())
                            .child(icon(Icon::Lock, 13., COLORS.primary))
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
                    .child(
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .spacing(8.)
                            .child(caption("SLOW DOWN. SETTLE IN."))
                            .child(
                                label()
                                    .text("There’s a little more to every day.")
                                    .font_family("DM Serif Display")
                                    .font_size(31.),
                            )
                            .child(
                                label()
                                    .text("A thought, a feeling, a small good thing. Keep it here.")
                                    .font_size(14.)
                                    .color(COLORS.text_secondary),
                            ),
                    )
                    .maybe_child(state.error.as_ref().map(|error| {
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .padding(14.)
                            .spacing(10.)
                            .background(Color::from_rgb(61, 35, 32))
                            .corner_radius(8.)
                            .child(prose(error.clone(), 13., Color::from_rgb(255, 196, 180)))
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
                    .maybe(state.privacy_open, |el| el.child(privacy_panel(journal)))
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
                                            .color(COLORS.primary),
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
                                    .background(COLORS.surface_secondary)
                                    .corner_radius(14.)
                                    .border(Border::new().width(1.).fill(COLORS.border))
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
                                                                    COLORS.text_secondary,
                                                                ))
                                                                .child(
                                                                    label()
                                                                        .text("Delete")
                                                                        .font_size(12.)
                                                                        .color(
                                                                            COLORS.text_secondary,
                                                                        ),
                                                                ),
                                                        ),
                                                )
                                            }),
                                    )
                                    .maybe(state.delete_pending, |el| {
                                        el.child(delete_confirmation(journal))
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
                                                            COLORS.error
                                                        } else {
                                                            COLORS.primary
                                                        },
                                                    ))
                                                    .child(caption(if unsaved {
                                                        "Not saved · your draft is kept open"
                                                    } else if state.draft.id.is_some() {
                                                        "Saved to this device"
                                                    } else {
                                                        "A fresh page · saves as you write"
                                                    })),
                                            )
                                            .child(caption(format!(
                                                "{words} {}",
                                                if words == 1 { "word" } else { "words" }
                                            ))),
                                    ),
                            ),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .padding((0., 0., 0., 82.))
                            .child(prompt()),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .max_width(Size::px(960.))
                            .horizontal()
                            .main_align(Alignment::space_between())
                            .child(caption(
                                "Not every day needs to be extraordinary to be worth remembering.",
                            ))
                            .child(caption(if cfg!(target_os = "macos") {
                                "⌘ N  new entry  ·  ⌘ F  search"
                            } else {
                                "Ctrl N  new entry  ·  Ctrl F  search"
                            })),
                    ),
            )
    }
}

fn delete_confirmation(mut journal: State<Journal>) -> impl IntoElement {
    rect()
        .width(Size::fill())
        .padding(12.)
        .spacing(10.)
        .background(COLORS.tertiary)
        .corner_radius(8.)
        .child(prose(
            "Delete this entry permanently? This cannot be undone.",
            13.,
            COLORS.text_primary,
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

fn privacy_panel(mut journal: State<Journal>) -> impl IntoElement {
    let path = journal.read().store.path.display().to_string();
    rect().width(Size::fill()).max_width(Size::px(960.)).padding(18.).spacing(8.)
        .background(COLORS.tertiary).corner_radius(10.)
        .child(label().text("Your words stay here.").font_size(16.).color(COLORS.primary))
        .child(prose("Daymark has no account, telemetry, sync, or network requests. The SQLite database is local, but NOT encrypted. Use OS disk encryption for protection at rest. Close Daymark before copying the database for a backup.", 13., COLORS.text_primary))
        .child(prose(path, 12., COLORS.text_secondary))
        .child(Action { text: "Back to writing", icon: Icon::Close, primary: false,
            on_press: (move |()| journal.write().privacy_open = false).into() })
}

fn prompt() -> impl IntoElement {
    rect()
        .width(Size::fill())
        .padding((18., 22.))
        .spacing(7.)
        .background(COLORS.surface_tertiary)
        .corner_radius(10.)
        .child(
            label()
                .text("A SMALL INVITATION")
                .font_size(10.)
                .color(COLORS.primary)
                .font_weight(FontWeight::MEDIUM),
        )
        .child(prose(
            "What’s one small thing you’d like to remember about today?",
            14.,
            COLORS.text_secondary,
        ))
}
