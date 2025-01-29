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
        self.class = (Box::new(style) as widget::button::StyleFn<'a, iced::Theme>).into();
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

const SUBMIT_SVG: &[u8] = include_bytes!("../../assets/check-lg.svg");
pub fn submit_button<'a, Message: Clone + 'a>(
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(widget::svg::Handle::from_memory(SUBMIT_SVG), "Save")
        .on_press_maybe(message)
        .style(widget::button::success)
        .into()
}

const CANCEL_SVG: &[u8] = include_bytes!("../../assets/x-lg.svg");
pub fn cancel_button<'a, Message: Clone + 'a>(
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(widget::svg::Handle::from_memory(CANCEL_SVG), "Cancel")
        .on_press_maybe(message)
        .style(widget::button::danger)
        .into()
}

const EDIT_SVG: &[u8] = include_bytes!("../../assets/pencil-square.svg");
pub fn edit_button<'a, Message: Clone + 'a>(
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(widget::svg::Handle::from_memory(EDIT_SVG), "Edit")
        .on_press_maybe(message)
        .style(widget::button::secondary)
        .into()
}

const DELETE_SVG: &[u8] = include_bytes!("../../assets/trash-fill.svg");
pub fn delete_button<'a, Message: Clone + 'a>(
    message: Option<Message>,
) -> iced::Element<'a, Message> {
    SvgButton::new(widget::svg::Handle::from_memory(DELETE_SVG), "Delete")
        .on_press_maybe(message)
        .style(widget::button::danger)
        .into()
}
