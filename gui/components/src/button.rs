use iced::widget;
use iced::widget::button::Catalog;

pub struct SvgButton<'a, Message> {
    svg: widget::svg::Handle,
    content: iced::Element<'a, Message>,
    on_press: Option<Message>,
    class: <iced::Theme as Catalog>::Class<'a>,
}

impl<'a, Message: 'a + Clone> SvgButton<'a, Message> {
    pub fn new(svg: widget::svg::Handle, content: impl Into<iced::Element<'a, Message>>) -> Self {
        Self {
            svg,
            content: content.into(),
            on_press: None,
            class: <iced::Theme as Catalog>::default(),
        }
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub fn style(
        mut self,
        style: impl Fn(&iced::Theme, widget::button::Status) -> widget::button::Style + 'a,
    ) -> Self {
        self.class = Box::new(style) as widget::button::StyleFn<'a, iced::Theme>;
        self
    }

    pub fn view(self) -> iced::Element<'a, Message> {
        widget::button(super::spal_row![
            widget::svg::Svg::new(self.svg).width(iced::Shrink),
            self.content
        ])
        .on_press_maybe(self.on_press)
        .class(self.class)
        .into()
    }
}

impl<'a, Message: 'a + Clone> From<SvgButton<'a, Message>> for iced::Element<'a, Message> {
    fn from(value: SvgButton<'a, Message>) -> Self {
        value.view()
    }
}

pub fn submit<'a, Message: Clone + 'a>(message: Option<Message>) -> iced::Element<'a, Message> {
    SvgButton::new(icons::CHECK_LG.clone(), "Save")
        .on_press_maybe(message)
        .style(widget::button::success)
        .into()
}

pub fn cancel<'a, Message: Clone + 'a>(message: Option<Message>) -> iced::Element<'a, Message> {
    SvgButton::new(icons::X_LG.clone(), "Cancel")
        .on_press_maybe(message)
        .style(widget::button::danger)
        .into()
}

pub fn edit<'a, Message: Clone + 'a>(message: Option<Message>) -> iced::Element<'a, Message> {
    edit_with_text("Edit", message)
}

pub fn edit_with_text<'a, Message: Clone + 'a>(
    text: &'a str,
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(icons::PENCIL_SQUARE.clone(), text)
        .on_press_maybe(message)
        .style(widget::button::secondary)
        .into()
}

pub fn delete<'a, Message: Clone + 'a>(message: Option<Message>) -> iced::Element<'a, Message> {
    SvgButton::new(icons::TRASH_FILL.clone(), "Delete")
        .on_press_maybe(message)
        .style(widget::button::danger)
        .into()
}

pub fn new<'a, Message: Clone + 'a>(
    content: &'a str,
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(icons::PLUS_SQUARE_FILL.clone(), content)
        .on_press_maybe(message)
        .into()
}

/// A large round plus button with a big margin.
/// It is supposed to be an overlay button to create new stuff.
pub fn large_round_plus_button<Message: Clone + 'static>(
    on_pressed_maybe: Option<Message>,
) -> iced::Element<'static, Message> {
    widget::container(
        widget::button(icons::plus_circle_fill())
            .style(|theme, status| {
                let mut style = widget::button::success(theme, status);
                style.border.radius = iced::border::Radius::new(100);
                style
            })
            .padding(15)
            .on_press_maybe(on_pressed_maybe),
    )
    .padding(15)
    .into()
}

pub fn right_attached_button<'a, Message: Clone>(
    content: impl Into<iced::Element<'a, Message>>,
) -> widget::Button<'a, Message> {
    widget::button(content)
        .style(|theme, status| {
            let mut style = widget::button::primary(theme, status);
            style.border.radius = style.border.radius.right(15.0);
            style
        })
        .padding(iced::Padding::new(5.0).right(10.0))
}
