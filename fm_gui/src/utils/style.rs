use iced::widget;

pub fn container_style_background_weak(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().with_background(iced::Background::Color(
        theme.extended_palette().background.weak.color,
    ))
}

pub fn container_style_background_strong(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().with_background(iced::Background::Color(
        theme.extended_palette().background.strong.color,
    ))
}

pub fn button_link_style(
    theme: &iced::Theme,
    status: widget::button::Status,
) -> widget::button::Style {
    widget::button::Style {
        background: None,
        text_color: if status == widget::button::Status::Hovered {
            theme.palette().text.scale_alpha(0.5)
        } else {
            theme.palette().text
        },
        ..Default::default()
    }
}
