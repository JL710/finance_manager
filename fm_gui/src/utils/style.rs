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
