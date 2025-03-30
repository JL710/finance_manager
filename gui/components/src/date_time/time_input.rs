use iced::widget;

use super::Shift;

#[derive(Debug, Clone)]
pub enum Message {
    OnInput(String),
    ToggleDropdown,
    TimeShift { shift: Shift, positive: bool },
}

#[derive(Debug, Default)]
pub struct State {
    value: String,
    drop_down: bool,
}

impl State {
    pub fn new(default: Option<time::Time>) -> Self {
        Self {
            value: if let Some(t) = default {
                super::to_time_string(t)
            } else {
                String::default()
            },
            drop_down: false,
        }
    }

    pub fn perform(&mut self, message: Message) {
        match message {
            Message::OnInput(input) => self.value = input,
            Message::ToggleDropdown => self.drop_down = !self.drop_down,
            Message::TimeShift { shift, positive } => {
                if let Some(before) = self.time() {
                    self.value = super::to_time_string(
                        super::apply_shift(
                            time::OffsetDateTime::now_utc().replace_time(before),
                            shift,
                            positive,
                        )
                        .time(),
                    )
                }
            }
        }
    }

    /// Hour and minute
    pub fn time(&self) -> Option<time::Time> {
        super::parse_time_str(&self.value).ok()
    }

    pub fn anything_entered(&self) -> bool {
        !self.value.is_empty()
    }
}

pub struct TimeInput<'a> {
    state: &'a State,
    required: bool,
}

impl<'a> TimeInput<'a> {
    pub fn new(state: &'a State, required: bool) -> Self {
        Self { state, required }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn view(self) -> iced::Element<'a, Message> {
        crate::drop_down(
            crate::centered_row![
                widget::text_input("12:00", &self.state.value)
                    .style(move |theme: &iced::Theme, status| {
                        let mut original = iced::widget::text_input::default(theme, status);

                        if (self.state.value.is_empty() && self.required)
                            || (!self.state.value.is_empty() && self.state.time().is_none())
                        {
                            original.border.color = theme.palette().danger;
                        } else if !self.state.value.is_empty() && self.state.time().is_some() {
                            original.border.color = theme.palette().success;
                        }
                        original
                    })
                    .on_input(Message::OnInput)
                    .width(60.0),
                crate::button::right_attached_button(
                    widget::Svg::new(widget::svg::Handle::from_memory(include_bytes!(
                        "../../../assets/pencil-fill.svg"
                    )))
                    .width(iced::Shrink)
                )
                .on_press_maybe(if self.state.time().is_some() {
                    Some(Message::ToggleDropdown)
                } else {
                    None
                })
            ],
            crate::spal_column![
                crate::spal_row![
                    widget::button("-").on_press(Message::TimeShift {
                        shift: Shift::Duration(time::Duration::MINUTE),
                        positive: false
                    }),
                    "Minute",
                    widget::button("+").on_press(Message::TimeShift {
                        shift: Shift::Duration(time::Duration::MINUTE),
                        positive: true
                    })
                ],
                crate::spal_row![
                    widget::button("-").on_press(Message::TimeShift {
                        shift: Shift::Duration(time::Duration::HOUR),
                        positive: false
                    }),
                    "Hour",
                    widget::button("+").on_press(Message::TimeShift {
                        shift: Shift::Duration(time::Duration::HOUR),
                        positive: true
                    })
                ]
            ],
            self.state.drop_down,
        )
        .on_dismiss(Message::ToggleDropdown)
        .into()
    }
}

pub fn time_input(state: &State, required: bool) -> TimeInput<'_> {
    TimeInput::new(state, required)
}
