use chrono::TimeZone;
use iced::widget;

pub fn labeled_entry<'a, Message: 'a + Clone>(
    name: &'a str,
    content: &str,
    message: impl Fn(String) -> Message + 'a,
) -> iced::Element<'a, Message> {
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
) -> iced::Element<'static, Message> {
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
/// - `amount_color`: A function that takes a transaction and returns a boolean that indicates how the amount should be colored
///     - `true`: The amount should be colored in a positive color
///     - `false`: The amount should be colored in a negative color
///     - `None`: The amount should not be colored
/// - `view_transaction`: A function that takes a transaction id and returns a message that will be produced when the transaction is clicked
/// - `view_account`: A function that takes an account id and returns a message that will be produced when the account is clicked
pub fn transaction_table<'a, Message: 'a + Clone>(
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    amount_color: impl Fn(fm_core::Transaction) -> Option<bool> + 'a,
    view_transaction: impl Fn(fm_core::Id) -> Message + 'static,
    view_account: impl Fn(fm_core::Id) -> Message + 'static,
) -> iced::Element<'a, Message> {
    let table = super::table_view::TableView::new(
        transactions.clone(),
        move |(transaction, source, destination): &(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )| {
            [
                widget::button(widget::text(transaction.title().clone()))
                    .on_press(view_transaction(*transaction.id()))
                    .style(button_link_style)
                    .padding(0)
                    .into(),
                widget::text(transaction.date().format("%d.%m.%Y").to_string()).into(),
                match amount_color(transaction.clone()) {
                    Some(true) => colored_currency_display(&transaction.amount()),
                    Some(false) => colored_currency_display(&transaction.amount().negative()),
                    None => widget::text(transaction.amount().to_string()).into(),
                },
                widget::button(widget::text(source.to_string().clone()))
                    .on_press(view_account(*source.id()))
                    .style(button_link_style)
                    .padding(0)
                    .into(),
                widget::button(widget::text(destination.to_string().clone()))
                    .on_press(view_account(*destination.id()))
                    .style(button_link_style)
                    .padding(0)
                    .into(),
            ]
        },
    )
    .headers([
        "Title".to_owned(),
        "Date".to_owned(),
        "Amount".to_owned(),
        "Source".to_owned(),
        "Destination".to_owned(),
    ])
    .columns_sortable([false, true, true, false, false])
    .sort_by(|a, b, column_index| match column_index {
        1 => a.0.date().cmp(&b.0.date()),
        2 => a.0.amount().get_num().total_cmp(&b.0.amount().get_num()),
        _ => std::cmp::Ordering::Equal,
    });

    table.into_element()
}
