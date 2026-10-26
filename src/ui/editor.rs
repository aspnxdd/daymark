use freya::{prelude::*, text_edit::*};

use super::theme::COLORS;
use crate::model::Journal;

#[derive(PartialEq)]
pub struct WritingArea {
    pub journal: State<Journal>,
    pub key: DiffKey,
}

impl KeyExt for WritingArea {
    fn key_mut(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for WritingArea {
    fn render_key(&self) -> DiffKey {
        self.key.clone()
    }

    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
        let title = use_state(|| journal.peek().draft.title.clone());
        rect()
            .height(Size::flex(1.))
            .width(Size::fill())
            .content(Content::flex())
            .spacing(22.)
            .child(
                rect().font_family("DM Serif Display").font_size(32.).child(
                    Input::new(title)
                        .on_pre_key_down(single_line_key_policy)
                        .width(Size::fill())
                        .placeholder("Give this day a title…")
                        .background(COLORS.surface_secondary)
                        .focus_background(COLORS.surface_secondary)
                        .border_fill(Color::TRANSPARENT)
                        .placeholder_color(COLORS.text_placeholder)
                        .padding((4., 0.))
                        .on_validate(move |value: InputValidator| {
                            journal.write().edit_title(value.text().to_string())
                        }),
                ),
            )
            .child(BodyEditor { journal })
    }
}

#[derive(PartialEq)]
struct BodyEditor {
    journal: State<Journal>,
}

impl Component for BodyEditor {
    fn render(&self) -> impl IntoElement {
        let mut journal = self.journal;
        let holder = use_state(ParagraphHolder::default);
        let mut editable = use_editable(|| journal.peek().draft.body.clone(), EditableConfig::new);
        let a11y = use_a11y();
        let focus = use_focus(a11y);
        let text = editable.editor().read().to_string();
        let empty = text.is_empty();
        let focused = focus().is_focused();

        ScrollView::new()
            .width(Size::fill())
            .height(Size::flex(1.))
            .scroll_with_arrows(false)
            .drag_scrolling(false)
            .child(
                rect()
                    .width(Size::fill())
                    .min_height(Size::px(300.))
                    .a11y_id(a11y)
                    .a11y_focusable(true)
                    .a11y_role(AccessibilityRole::MultilineTextInput)
                    .a11y_alt("Journal entry. Changes save automatically to this device.")
                    .cursor(CursorIcon::Text)
                    .border(Border::new().width(1.).fill(if focus() == Focus::Keyboard {
                        COLORS.border_focus
                    } else {
                        Color::TRANSPARENT
                    }))
                    .corner_radius(4.)
                    .on_focus_press(move |e: Event<FocusPressEventData>| {
                        e.stop_propagation();
                        a11y.request_focus();
                    })
                    .on_key_down(move |e: Event<KeyboardEventData>| {
                        if matches!(e.key, Key::Named(NamedKey::Tab)) {
                            return;
                        }
                        if matches!(e.key, Key::Named(NamedKey::Escape)) {
                            a11y.request_unfocus();
                            return;
                        }
                        if command_shortcut(&e) {
                            return;
                        }
                        e.stop_propagation();
                        editable.process_event(EditableEvent::KeyDown {
                            key: &e.key,
                            modifiers: e.modifiers,
                            editor_line: Some(EditorLine::SingleParagraph),
                            holder: Some(&holder.read()),
                        });
                        let text = editable.editor().peek().committed_text();
                        journal.write().edit_body(text);
                    })
                    .on_key_up(move |e: Event<KeyboardEventData>| {
                        editable.process_event(EditableEvent::KeyUp { key: &e.key });
                    })
                    .on_ime_preedit(move |e: Event<ImePreeditEventData>| {
                        let mut editor = editable.editor_mut().write();
                        if e.text.is_empty() {
                            editor.clear_preedit();
                        } else {
                            editor.set_preedit(&e.text);
                        }
                    })
                    .on_pointer_press(move |e: Event<PointerEventData>| {
                        // Match Input: an old input must not unfocus this newly focused editor.
                        e.stop_propagation();
                        e.prevent_default();
                        editable.process_event(EditableEvent::Release);
                    })
                    .on_global_pointer_up(move |_: Event<PointerEventData>| {
                        editable.process_event(EditableEvent::Release);
                    })
                    .child(
                        paragraph()
                            .width(Size::fill())
                            .min_height(Size::px(300.))
                            .font_size(17.)
                            .line_height(1.8)
                            .color(if empty {
                                COLORS.text_placeholder
                            } else {
                                COLORS.text_primary
                            })
                            .cursor_index(if focused {
                                Some(editable.editor().read().cursor_pos())
                            } else {
                                None
                            })
                            .cursor_color(COLORS.primary)
                            .highlight_color(COLORS.text_highlight)
                            .highlights(if focused {
                                editable.editor().read().get_selection().map(|s| vec![s])
                            } else {
                                None
                            })
                            .on_mouse_down(move |e: Event<MouseEventData>| {
                                a11y.request_focus();
                                if !empty {
                                    editable.process_event(EditableEvent::Down {
                                        location: e.element_location,
                                        editor_line: EditorLine::SingleParagraph,
                                        holder: &holder.read(),
                                    });
                                }
                            })
                            .on_mouse_move(move |e: Event<MouseEventData>| {
                                editable.process_event(EditableEvent::Move {
                                    location: e.element_location,
                                    editor_line: EditorLine::SingleParagraph,
                                    holder: &holder.read(),
                                });
                            })
                            .span(if empty {
                                "Start anywhere. What’s on your mind?".to_string()
                            } else {
                                text
                            })
                            .holder(holder.read().clone()),
                    ),
            )
    }
}

pub fn command_shortcut(event: &KeyboardEventData) -> bool {
    event.modifiers.contains(Modifiers::ctrl_or_meta())
        && matches!(&event.key, Key::Character(key) if ["n", "f", "s"].contains(&key.to_lowercase().as_str()))
}

pub fn single_line_key_policy(event: Event<KeyboardEventData>) -> bool {
    if command_shortcut(&event) {
        return false;
    }
    match &event.key {
        Key::Named(NamedKey::Tab) => false,
        Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Shift) => true,
        _ => {
            event.stop_propagation();
            event.prevent_default();
            true
        }
    }
}
