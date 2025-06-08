use anyhow::Context;
pub use iced::{self, widget};

pub mod button;
pub mod currency_input;
pub mod filter_component;
pub mod key_value_editor;
mod scrollable;
pub mod table_view;
pub mod transaction_table;

pub mod date_time;

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

    spal_row![name, input].into()
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
    .padding(iced::Padding::default().bottom(style::PADDING))
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

pub fn submit_cancel_row<'a, Message: Clone + 'a>(
    submit: Option<Message>,
    cancel: Option<Message>,
) -> iced::Element<'a, Message> {
    spal_row![
        button::cancel(cancel),
        widget::horizontal_space(),
        button::submit(submit),
    ]
    .into()
}

/// The column macro from iced with a default spacing
#[macro_export]
macro_rules! spaced_column {
    () => (
        $crate::widget::Column::new().spacing(style::COLUMN_SPACING)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Column::with_children([$($crate::iced::Element::from($x)),+]).spacing(style::COLUMN_SPACING)
    );
}

/// The row macro from iced with a default spacing
#[macro_export]
macro_rules! spaced_row {
    () => (
        $crate::widget::Row::new().spacing(style::ROW_SPACING)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Row::with_children([$($crate::iced::Element::from($x)),+]).spacing(style::ROW_SPACING)
    );
}

/// The default row macro from iced with a default padding and vertical centered layout.
///
/// Its initial purpose is to be used for inputs that have a label and some input.
///
/// spal stands for spaced and aligned
#[macro_export]
macro_rules! spal_row {
    () => (
        $crate::widget::Row::new()
            .spacing(style::ROW_SPACING)
            .align_y($crate::iced::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Row::with_children([$($crate::iced::Element::from($x)),+])
            .spacing(style::ROW_SPACING)
            .align_y($crate::iced::Alignment::Center)
    );
}

/// The default column macro from iced with a default padding and vertical centered layout.
///
/// spal stands for spaced and aligned
#[macro_export]
macro_rules! spal_column {
    () => (
        $crate::widget::Column::new()
            .spacing(style::COLUMN_SPACING)
            .align_x($crate::iced::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Column::with_children([$($crate::iced::Element::from($x)),+])
            .spacing(style::COLUMN_SPACING)
            .align_x($crate::iced::Alignment::Center)
    );
}

/// The column macro from iced with align center
#[macro_export]
macro_rules! centered_column {
    () => (
        $crate::widget::Column::new().align_x($crate::iced::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Column::with_children([$($crate::iced::Element::from($x)),+]).align_x($crate::iced::Alignment::Center)
    );
}

/// The row macro from iced with align center
#[macro_export]
macro_rules! centered_row {
    () => (
        $crate::widget::Row::new().align_y($crate::iced::Alignment::Center)
    );
    ($($x:expr),+ $(,)?) => (
        $crate::widget::Row::with_children([$($crate::iced::Element::from($x)),+]).align_y($crate::iced::Alignment::Center)
    );
}

/// Creates a dropdown where the width is set to shrink, and the overlay is placed in a bordered container.
pub fn drop_down<'a, Message: Clone + 'a>(
    underlay: impl Into<iced::Element<'a, Message>>,
    overlay: impl Into<iced::Element<'a, Message>>,
    expanded: bool,
) -> iced_aw::drop_down::DropDown<'a, Message> {
    iced_aw::drop_down::DropDown::new(
        underlay,
        style::container_popup_styling(widget::container(overlay)),
        expanded,
    )
    .width(iced::Shrink)
}

pub fn category_distribution_popup<Message: std::marker::Send + 'static>(
    finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    transaction_ids: Vec<fm_core::Id>,
    title: String,
    additional_description: Option<String>,
) -> iced::Task<Message> {
    error::failing_task(async move {
        let transactions = finance_controller.get_transactions(transaction_ids).await?;
        let category_distribution = fm_core::transactions_category_distribution(transactions);

        let mut displayed_text = String::new();
        if let Some(additional_description) = additional_description {
            displayed_text += &format!("{}\n\n", additional_description);
        }
        for (category_id, value) in category_distribution {
            displayed_text += &format!(
                "{}: {}\n",
                finance_controller
                    .get_category(category_id)
                    .await?
                    .context("Category not found")?
                    .name,
                value
            );
        }
        displayed_text += "\nCategory Distribution is a list of transactions summed by categories (and the sign for each). If a transaction is in multiple categories it will be in the sum of each of those categories.";

        rfd::AsyncMessageDialog::new()
            .set_title(title)
            .set_description(displayed_text)
            .set_buttons(rfd::MessageButtons::Ok)
            .show()
            .await;

        Ok(())
    })
    .discard()
}

pub fn overlap_bottom_right<'a, Message: Clone + 'static>(
    base: impl Into<iced::Element<'a, Message>>,
    overlay: impl Into<iced::Element<'a, Message>>,
) -> widget::Stack<'a, Message> {
    widget::stack![
        base.into(),
        widget::container(overlay.into())
            .align_right(iced::Fill)
            .align_bottom(iced::Fill)
    ]
}
