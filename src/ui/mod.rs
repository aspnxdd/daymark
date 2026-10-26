pub mod components;
mod editor;
mod sidebar;
pub mod theme;
mod workspace;

#[cfg(test)]
mod tests;

use std::cell::RefCell;

use chrono::{DateTime, Local, NaiveDate};
use freya::prelude::*;

use crate::model::Journal;
use sidebar::Sidebar;
use theme::ThemePreference;
use workspace::Workspace;

pub struct DaymarkApp {
    pub initial: RefCell<Option<Journal>>,
}

impl App for DaymarkApp {
    fn render(&self) -> impl IntoElement {
        let mut journal = use_state(|| {
            self.initial
                .borrow_mut()
                .take()
                .expect("app initializes once")
        });
        let preference = use_state(|| ThemePreference::load(&journal.peek().store.path));
        use_provide_context(|| preference);
        let theme = use_init_theme(|| preference.peek().selected.theme());
        let colors = theme.read().colors.clone();
        let search_id = use_a11y();
        let focus_mode = journal.read().focus_mode;
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .horizontal()
            .content(Content::flex())
            .font_family("DM Sans")
            .font_size(14.)
            .color(colors.text_primary)
            .background(colors.background)
            .on_global_key_down(move |e: Event<KeyboardEventData>| {
                if !e.modifiers.contains(Modifiers::ctrl_or_meta()) {
                    return;
                }
                if let Key::Character(key) = &e.key {
                    match key.to_lowercase().as_str() {
                        "n" => journal.write().new_entry(),
                        "f" => {
                            journal.write().focus_mode = false;
                            search_id.request_focus();
                        }
                        "s" => {
                            journal.write().save();
                        }
                        _ => {}
                    }
                }
            })
            .maybe(!focus_mode, |el| el.child(Sidebar { journal, search_id }))
            .child(Workspace { journal })
    }
}

fn entry_date(value: &str) -> NaiveDate {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.date_naive())
        .unwrap_or_else(|_| Local::now().date_naive())
}
