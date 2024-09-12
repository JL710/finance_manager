use iced::widget;

mod currency_input;
mod date_input;
mod filter_component;
pub mod style;
mod table_view;
mod timespan_input;

pub use currency_input::CurrencyInput;
pub use date_input::DateInput;
pub use filter_component::FilterComponent;
pub use table_view::TableView;
pub use timespan_input::TimespanInput;

pub fn labeled_entry<'a, Message: 'a + Clone>(
    name: &'a str,
    content: &str,
    message: impl Fn(String) -> Message + 'a,
    required: bool,
) -> iced::Element<'a, Message> {
    let mut input = widget::text_input(name, content).on_input(message);
    if required {
        if content.is_empty() {
            input = input.style(style::text_input_danger);
        } else {
            input = input.style(style::text_input_success);
        }
    }

    widget::row![widget::text(name), input].spacing(10).into()
}

pub fn parse_to_datetime(date: &str) -> anyhow::Result<time::OffsetDateTime> {
    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::parse(
            date,
            &time::format_description::parse("[day].[month].[year]")?,
        )?,
        time::Time::MIDNIGHT,
        fm_core::get_local_timezone()?,
    ))
}

pub fn colored_currency_display<Message>(
    value: &fm_core::Currency,
) -> iced::Element<'static, Message> {
    if value.get_eur_num() < 0.0 {
        widget::text!("{}", value)
            .style(|theme: &iced::Theme| widget::text::Style {
                color: Some(theme.palette().danger),
            })
            .into()
    } else {
        widget::text!("+{}", value)
            .style(|theme: &iced::Theme| widget::text::Style {
                color: Some(theme.palette().success),
            })
            .into()
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
    amount_positive: impl Fn(fm_core::Transaction) -> Option<bool> + 'a + Copy,
    view_transaction: impl Fn(fm_core::Id) -> Message + 'static,
    view_account: impl Fn(fm_core::Id) -> Message + 'static,
) -> iced::Element<'a, Message> {
    let mut transactions = transactions;
    transactions.sort_by(|(a, _, _), (b, _, _)| b.date().cmp(a.date()));
    let table = TableView::new(
        transactions.clone(),
        move |(transaction, source, destination): &(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )| {
            [
                link(widget::text(transaction.title().clone()))
                    .on_press(view_transaction(*transaction.id()))
                    .into(),
                widget::text(
                    transaction
                        .date()
                        .to_offset(fm_core::get_local_timezone().unwrap())
                        .format(&time::format_description::parse("[day].[month].[year]").unwrap())
                        .unwrap(),
                )
                .into(),
                match amount_positive(transaction.clone()) {
                    Some(true) => colored_currency_display(&transaction.amount()),
                    Some(false) => colored_currency_display(&transaction.amount().negative()),
                    None => widget::text(transaction.amount().to_string()).into(),
                },
                link(widget::text(source.to_string().clone()))
                    .on_press(view_account(*source.id()))
                    .into(),
                link(widget::text(destination.to_string().clone()))
                    .on_press(view_account(*destination.id()))
                    .into(),
                widget::text(transaction.categories().len().to_string()).into(),
            ]
        },
    )
    .headers([
        "Title".to_owned(),
        "Date".to_owned(),
        "Amount".to_owned(),
        "Source".to_owned(),
        "Destination".to_owned(),
        "Categories".to_owned(),
    ])
    .columns_sortable([true, true, true, true, true, true])
    .sort_by(move |a, b, column_index| match column_index {
        0 => a.0.title().cmp(b.0.title()),
        1 => a.0.date().cmp(b.0.date()),
        2 => {
            let a = amount_positive(a.0.clone()).map_or(a.0.amount(), |positive| {
                if positive {
                    a.0.amount()
                } else {
                    a.0.amount().negative()
                }
            });
            let b = amount_positive(b.0.clone()).map_or(b.0.amount(), |positive| {
                if positive {
                    b.0.amount()
                } else {
                    b.0.amount().negative()
                }
            });
            a.cmp(&b)
        }
        3 => a.1.name().cmp(b.1.name()),
        4 => a.2.name().cmp(b.2.name()),
        5 => a.0.categories().len().cmp(&b.0.categories().len()),
        _ => std::cmp::Ordering::Equal,
    });

    table.into()
}

pub fn link<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> widget::Button<'a, Message> {
    widget::button(content)
        .padding(0)
        .style(widget::button::text)
}

pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    pub fn text_size(&self) -> f32 {
        let default_size = iced::Settings::default().default_text_size;
        match self {
            HeadingLevel::H1 => default_size.0 + 20.,
            HeadingLevel::H2 => default_size.0 + 15.,
            HeadingLevel::H3 => default_size.0 + 10.,
            HeadingLevel::H4 => default_size.0 + 5.,
            HeadingLevel::H5 => default_size.0 + 3.,
            HeadingLevel::H6 => default_size.0 + 1.,
        }
    }
}

pub fn markdown_settings() -> widget::markdown::Settings {
    let mut settings = widget::markdown::Settings::default();
    settings.h1_size = HeadingLevel::H1.text_size().into();
    settings.h2_size = HeadingLevel::H2.text_size().into();
    settings.h3_size = HeadingLevel::H3.text_size().into();
    settings.h4_size = HeadingLevel::H4.text_size().into();
    settings.h5_size = HeadingLevel::H5.text_size().into();
    settings.h6_size = HeadingLevel::H6.text_size().into();
    settings
}

pub fn heading<'a, Message: 'a>(
    text: impl Into<String>,
    level: HeadingLevel,
) -> iced::Element<'a, Message> {
    widget::container(widget::column![
        widget::text(text.into())
            .size(level.text_size())
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        widget::horizontal_rule(2.).style(|theme: &iced::Theme| widget::rule::Style {
            color: theme.palette().text,
            ..widget::rule::default(theme)
        })
    ])
    .padding(iced::Padding {
        top: 0.,
        right: 0.,
        bottom: 10.,
        left: 0.,
    })
    .into()
}

pub fn parse_number(input: &str) -> Option<f64> {
    let input = input
        .replace(",", ".")
        .chars()
        .filter(|c| *c != '_')
        .collect::<String>();
    input.parse().ok()
}
