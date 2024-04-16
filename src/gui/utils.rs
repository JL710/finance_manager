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
