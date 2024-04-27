use chrono::TimeZone;
use iced::widget;

pub fn labeled_entry<'a, Message: 'a + Clone>(
    name: &'a str,
    content: &str,
    message: impl Fn(String) -> Message + 'a,
) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
    widget::row![
        widget::text(name),
        widget::text_input(name, content).on_input(message)
    ]
    .spacing(10)
    .into()
}

pub fn entry_row_container_style_weak(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().with_background(iced::Background::Color(
        theme.extended_palette().background.weak.color,
    ))
}

pub fn entry_row_container_style_strong(theme: &iced::Theme) -> widget::container::Style {
    widget::container::Style::default().with_background(iced::Background::Color(
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
        widget::text(format!("{}", value))
            .style(|theme: &iced::Theme| widget::text::Style {
                color: Some(theme.palette().danger),
            })
            .into()
    } else {
        widget::text(format!("+{}", value))
            .style(|theme: &iced::Theme| widget::text::Style {
                color: Some(theme.palette().success),
            })
            .into()
    }
}

pub fn button_link_style(
    theme: &iced::Theme,
    _status: widget::button::Status,
) -> widget::button::Style {
    widget::button::Style {
        background: None,
        text_color: theme.palette().text,
        ..Default::default()
    }
}

/// Create a table of transactions
///
/// # Arguments
/// - `transactions`: A slice of tuples of transactions and their source and destination accounts
/// - `view_transaction`: A function that takes a transaction id and returns a message that will be produced when the transaction is clicked
/// - `view_account`: A function that takes an account id and returns a message that will be produced when the account is clicked
pub fn transaction_table<'a, Message: 'a + Clone>(
    transactions: &'a [(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )],
    view_transaction: impl Fn(fm_core::Id) -> Message,
    view_account: impl Fn(fm_core::Id) -> Message,
) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
    let mut transaction_table = super::table::Table::new(5).set_headers(vec![
        "Title".to_owned(),
        "Date".to_owned(),
        "Amount".to_owned(),
        "Source".to_owned(),
        "Destination".to_owned(),
    ]);
    for transaction in transactions {
        transaction_table.push_row(vec![
            widget::button(transaction.0.title().as_str())
                .style(button_link_style)
                .on_press(view_transaction(*transaction.0.id()))
                .padding(0)
                .into(),
            widget::text(transaction.0.date().format("%d.%m.%Y").to_string()).into(),
            widget::text(transaction.0.amount().to_string()).into(),
            widget::button(widget::text(transaction.1.to_string()))
                .on_press(view_account(transaction.1.id()))
                .style(button_link_style)
                .padding(0)
                .into(),
            widget::button(widget::text(transaction.2.to_string()))
                .on_press(view_account(transaction.2.id()))
                .style(button_link_style)
                .padding(0)
                .into(),
        ]);
    }
    transaction_table.convert_to_view()
}
