use anyhow::Result;
use std::future::Future;

pub use iced::{self, widget};

pub mod button;
pub mod currency_input;
pub mod filter_component;
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

pub fn error_chain_string(error: anyhow::Error) -> String {
    let mut message = String::new();

    let mut chain_iter = error.chain();

    message += &format!("Error: {}\n", chain_iter.next().unwrap());

    for err in chain_iter {
        message += &format!("\nCaused by:\n\t{}", err);
    }

    message
}

pub async fn error_popup(description: String) {
    rfd::AsyncMessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_title("An Error has occurred")
        .set_level(rfd::MessageLevel::Error)
        .set_description(description)
        .show()
        .await;
}

pub async fn async_popup_wrapper<T>(fut: impl Future<Output = Result<T>>) -> Option<T> {
    match fut.await {
        Err(error) => {
            error_popup(error_chain_string(error)).await;
            None
        }
        Ok(x) => Some(x),
    }
}

pub fn failing_task<T: Send + 'static>(
    fut: impl Future<Output = Result<T>> + Send + 'static,
) -> iced::Task<T> {
    iced::Task::future(async { async_popup_wrapper(fut).await }).and_then(iced::Task::done)
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

pub fn right_attached_button<'a, Message: Clone>(
    content: impl Into<iced::Element<'a, Message>>,
) -> widget::Button<'a, Message> {
    widget::button(content)
        .style(|theme, status| {
            let mut style = widget::button::primary(theme, status);
            style.border.radius = style.border.radius.right(15.0);
            style
        })
        .padding(iced::Padding::new(5.0).right(10.0))
}
