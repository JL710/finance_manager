use chrono::TimeZone;

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

pub fn entry_row_container_style_header(theme: &iced::Theme) -> iced::widget::container::Style {
    match theme {
        iced::Theme::Dark => iced::widget::container::Style::default()
            .with_background(iced::Background::Color(iced::Color::from_rgb8(80, 80, 80))),
        _ => iced::widget::container::Style::default()
            .with_background(iced::Background::Color(iced::Color::from_rgb8(80, 80, 80))),
    }
}

pub fn parse_to_datetime(date: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    let date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Ok(chrono::Utc.from_utc_datetime(&date))
}
