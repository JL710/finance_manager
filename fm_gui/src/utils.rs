pub fn labeled_entry<'a, Message: 'a + Clone>(
    name: &'a str,
    content: &str,
    message: impl Fn(String) -> Message + 'a,
) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
    iced::widget::row![
        iced::widget::text(name),
        iced::widget::text_input(name, content).on_input(message)
    ]
    .spacing(10)
    .into()
}

pub fn entry_row_container_style(theme: &iced::Theme) -> iced::widget::container::Style {
    match theme {
        iced::Theme::Dark => iced::widget::container::Style::default().with_background(
            iced::Background::Color(iced::Color::from_rgb8(100, 100, 100)),
        ),
        _ => iced::widget::container::Style::default().with_background(iced::Background::Color(
            iced::Color::from_rgb8(100, 100, 100),
        )),
    }
}
