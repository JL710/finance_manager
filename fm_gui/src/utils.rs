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

pub fn entry_row_container_style_weak(theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default().with_background(iced::Background::Color(
        theme.extended_palette().background.weak.color,
    ))
}

pub fn entry_row_container_style_strong(theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style::default().with_background(iced::Background::Color(
        theme.extended_palette().background.strong.color,
    ))
}

pub fn parse_to_datetime(date: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    let date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Ok(chrono::Utc.from_utc_datetime(&date))
}

pub fn colored_currency_display<Message>(
    value: &fm_core::Currency,
) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
    if value.get_num() < 0.0 {
        iced::widget::text(format!("{}", value))
            .style(|theme: &iced::Theme| iced::widget::text::Style {
                color: Some(theme.palette().danger),
            })
            .into()
    } else {
        iced::widget::text(format!("+{}", value))
            .style(|theme: &iced::Theme| iced::widget::text::Style {
                color: Some(theme.palette().success),
            })
            .into()
    }
}
