use freya::{icons::lucide, prelude::*};

#[derive(Clone, Copy, PartialEq)]
pub enum Icon {
    Leaf,
    Plus,
    Search,
    Lock,
    Check,
    Focus,
    Trash,
    Close,
    Arrow,
}

pub fn icon(kind: Icon, size: f32, color: Color) -> SvgViewer {
    let source = match kind {
        Icon::Leaf => ("leaf", lucide::leaf()),
        Icon::Plus => ("plus", lucide::plus()),
        Icon::Search => ("search", lucide::search()),
        Icon::Lock => ("lock", lucide::lock()),
        Icon::Check => ("check", lucide::check()),
        Icon::Focus => ("maximize", lucide::maximize()),
        Icon::Trash => ("trash-2", lucide::trash_2()),
        Icon::Close => ("x", lucide::x()),
        Icon::Arrow => ("arrow-up-right", lucide::arrow_up_right()),
    };
    SvgViewer::new(source)
        .color(color)
        .width(Size::px(size))
        .height(Size::px(size))
}

pub fn caption(text: impl Into<String>) -> impl IntoElement {
    Caption { text: text.into() }
}

#[derive(PartialEq)]
struct Caption {
    text: String,
}

impl Component for Caption {
    fn render(&self) -> impl IntoElement {
        let colors = use_theme().read().colors.clone();
        label()
            .text(self.text.clone())
            .font_size(11.)
            .color(colors.text_secondary)
    }
}

pub fn divider() -> impl IntoElement {
    Divider
}

#[derive(PartialEq)]
struct Divider;

impl Component for Divider {
    fn render(&self) -> impl IntoElement {
        let colors = use_theme().read().colors.clone();
        rect()
            .width(Size::fill())
            .height(Size::px(1.))
            .background(colors.border)
    }
}

#[derive(PartialEq)]
pub struct Action {
    pub text: &'static str,
    pub icon: Icon,
    pub primary: bool,
    pub on_press: EventHandler<()>,
}

impl Component for Action {
    fn render(&self) -> impl IntoElement {
        let on_press = self.on_press.clone();
        let colors = use_theme().read().colors.clone();
        let color = if self.primary {
            colors.text_inverse
        } else {
            colors.text_secondary
        };
        Button::new()
            .background(if self.primary {
                colors.primary
            } else {
                colors.surface_secondary
            })
            .hover_background(if self.primary {
                colors.primary
            } else {
                colors.active
            })
            .focus_border_fill(if self.primary {
                colors.text_inverse
            } else {
                colors.border_focus
            })
            .color(color)
            .border_fill(if self.primary {
                colors.primary
            } else {
                colors.border
            })
            .corner_radius(8.)
            .padding((11., 16.))
            .on_press(move |_| on_press.call(()))
            .child(
                rect()
                    .horizontal()
                    .cross_align(Alignment::center())
                    .spacing(9.)
                    .child(icon(self.icon, 16., color))
                    .child(
                        label()
                            .text(self.text)
                            .font_size(13.)
                            .font_weight(FontWeight::MEDIUM),
                    ),
            )
    }
}

pub fn prose(text: impl Into<String>, size: f32, color: Color) -> impl IntoElement {
    paragraph()
        .width(Size::fill())
        .font_size(size)
        .line_height(1.55)
        .color(color)
        .span(text.into())
}
