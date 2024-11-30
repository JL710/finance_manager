use iced::widget;

pub fn container_style_background_weak(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().background(iced::Background::Color(
        theme.extended_palette().background.weak.color,
    ))
}

pub fn container_style_background_strong(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().background(iced::Background::Color(
        theme.extended_palette().background.strong.color,
    ))
}

pub fn container_popup_styling<Message>(
    container: widget::Container<'_, Message>,
) -> widget::Container<'_, Message> {
    container
        .style(|theme| {
            let mut style = container_style_background_strong(theme);
            style.border.radius = 10.0.into();
            style.border.color = theme.palette().primary;
            style.border.width = 3.0;
            style
        })
        .padding(10)
}

pub fn button_sidebar(
    theme: &iced::Theme,
    status: widget::button::Status,
) -> widget::button::Style {
    let mut style = widget::button::text(theme, status);
    style.text_color = match status {
        widget::button::Status::Active => theme.extended_palette().primary.strong.color,
        widget::button::Status::Disabled => theme.extended_palette().primary.weak.color,
        widget::button::Status::Hovered => theme.extended_palette().primary.base.color,
        widget::button::Status::Pressed => {
            theme.extended_palette().primary.base.color.scale_alpha(0.9)
        }
    };
    style
}

pub fn text_input_danger(
    theme: &iced::Theme,
    status: widget::text_input::Status,
) -> widget::text_input::Style {
    let mut style = widget::text_input::default(theme, status);
    style.border.color = theme.palette().danger;
    style
}

pub fn text_input_primary(
    theme: &iced::Theme,
    status: widget::text_input::Status,
) -> widget::text_input::Style {
    let mut style = widget::text_input::default(theme, status);
    style.border.color = theme.palette().primary;
    style
}

pub fn text_input_success(
    theme: &iced::Theme,
    status: widget::text_input::Status,
) -> widget::text_input::Style {
    let mut style = widget::text_input::default(theme, status);
    style.border.color = theme.palette().success;
    style
}
