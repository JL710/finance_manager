use iced::widget;

pub mod currency_input;
pub mod date_input;
pub mod filter_component;
pub mod style;
pub mod table_view;
pub mod timespan_input;
pub mod transaction_table;

pub use transaction_table::TransactionTable;

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
    let mut splits = date
        .replace("/", ".")
        .split(".")
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if splits.len() != 3 {
        anyhow::bail!("Illegal date format");
    }
    if splits[0].len() == 1 {
        splits[0] = format!("0{}", splits[0]);
    }
    if splits[1].len() == 1 {
        splits[1] = format!("0{}", splits[1]);
    }

    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::parse(
            format!("{}.{}.{}", splits[0], splits[1], splits[2]).as_str(),
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
    widget::markdown::Settings {
        h1_size: HeadingLevel::H1.text_size().into(),
        h2_size: HeadingLevel::H2.text_size().into(),
        h3_size: HeadingLevel::H3.text_size().into(),
        h4_size: HeadingLevel::H4.text_size().into(),
        h5_size: HeadingLevel::H5.text_size().into(),
        h6_size: HeadingLevel::H6.text_size().into(),
        ..Default::default()
    }
}

pub fn heading<'a, Message: 'a>(
    text: impl Into<String>,
    level: HeadingLevel,
) -> iced::Element<'a, Message> {
    widget::container(widget::column![
        {
            if cfg!(target_arch = "wasm32") {
                widget::text(text.into()).size(level.text_size())
            } else {
                widget::text(text.into())
                    .size(level.text_size())
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
            }
        },
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

fn modal<'a, Message>(
    base: impl Into<iced::Element<'a, Message>>,
    content: impl Into<iced::Element<'a, Message>>,
    on_blur: Message,
    open: bool,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
{
    widget::stack![base.into(),]
        .push_maybe(if open {
            Some(widget::opaque(
                widget::mouse_area(widget::center(widget::opaque(content)).style(|_theme| {
                    widget::container::Style {
                        background: Some(
                            iced::Color {
                                a: 0.8,
                                ..iced::Color::BLACK
                            }
                            .into(),
                        ),
                        ..widget::container::Style::default()
                    }
                }))
                .on_press(on_blur),
            ))
        } else {
            None
        })
        .into()
}
