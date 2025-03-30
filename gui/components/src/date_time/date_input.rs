use super::Shift;
use iced::widget;

#[derive(Default, Debug, Clone)]
pub struct State {
    value: String,
    default_value: Option<fm_core::DateTime>,
    drop_down: bool,
}

impl State {
    pub fn new(value: Option<fm_core::DateTime>) -> Self {
        Self {
            value: if let Some(date) = value {
                super::to_date_string(date)
            } else {
                String::new()
            },
            default_value: None,
            drop_down: false,
        }
    }

    pub fn new_with_raw(value: String) -> Self {
        Self {
            value,
            default_value: None,
            drop_down: false,
        }
    }

    pub fn default_value(&mut self, value: Option<fm_core::DateTime>) {
        self.default_value = value;
    }

    /// Will return the user input as datetime if possible or default datetime if given.
    ///
    /// The time will be set o 12:00.
    pub fn date(&self) -> Option<fm_core::DateTime> {
        super::parse_date_str(&self.value, 12, 0, 0).map_or(self.default_value, Some)
    }

    fn raw_input(&self) -> &str {
        &self.value
    }

    pub fn anything_entered(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::InputChanged(new_value) => {
                self.value = new_value;
                self.drop_down = false
            }
            Action::ToggleDropdown => self.drop_down = !self.drop_down,
            Action::DateShift { shift, positive } => {
                if let Some(before) = self.date() {
                    self.value = super::to_date_string(super::apply_shift(before, shift, positive))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    InputChanged(String),
    ToggleDropdown,
    DateShift { shift: Shift, positive: bool },
}

pub struct DateInput<'a> {
    required: bool,
    state: &'a State,
    placeholder: String,
}

impl<'a> DateInput<'a> {
    pub fn new(state: &'a State, placeholder: String, required: bool) -> Self {
        DateInput {
            state,
            required,
            placeholder,
        }
    }

    pub fn view(self) -> iced::Element<'a, Action> {
        crate::drop_down(
            crate::centered_row![
                widget::text_input(&self.placeholder, self.state.raw_input())
                    .on_input(Action::InputChanged)
                    .style(move |theme: &iced::Theme, status| {
                        let mut original = iced::widget::text_input::default(theme, status);

                        if (self.state.raw_input().is_empty() && self.required)
                            || (!self.state.raw_input().is_empty() && self.state.date().is_none())
                        {
                            original.border.color = theme.palette().danger;
                        } else if !self.state.raw_input().is_empty() && self.state.date().is_some()
                        {
                            original.border.color = theme.palette().success;
                        }
                        original
                    })
                    .width(100.0),
                crate::button::right_attached_button(
                    widget::Svg::new(widget::svg::Handle::from_memory(include_bytes!(
                        "../../../assets/pencil-fill.svg"
                    )))
                    .width(iced::Shrink)
                )
                .on_press_maybe(if self.state.date().is_some() {
                    Some(Action::ToggleDropdown)
                } else {
                    None
                })
            ],
            crate::spal_column![
                crate::spal_row![
                    widget::button("-").on_press(Action::DateShift {
                        shift: Shift::Duration(time::Duration::DAY),
                        positive: false
                    }),
                    "Day",
                    widget::button("+").on_press(Action::DateShift {
                        shift: Shift::Duration(time::Duration::DAY),
                        positive: true
                    })
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::DateShift {
                        shift: Shift::Duration(time::Duration::WEEK),
                        positive: false
                    }),
                    "Week",
                    widget::button("+").on_press(Action::DateShift {
                        shift: Shift::Duration(time::Duration::WEEK),
                        positive: true
                    })
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::DateShift {
                        shift: Shift::Month,
                        positive: false
                    }),
                    "Month",
                    widget::button("+").on_press(Action::DateShift {
                        shift: Shift::Month,
                        positive: true
                    })
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::DateShift {
                        shift: Shift::Year,
                        positive: false
                    }),
                    "Year",
                    widget::button("+").on_press(Action::DateShift {
                        shift: Shift::Year,
                        positive: true
                    })
                ]
            ],
            self.state.drop_down,
        )
        .on_dismiss(Action::ToggleDropdown)
        .into()
    }
}

pub fn date_input<'a>(state: &'a State, placeholder: &'a str, required: bool) -> DateInput<'a> {
    DateInput::new(state, placeholder.to_owned(), required)
}

impl<'a> From<DateInput<'a>> for iced::Element<'a, Action> {
    fn from(value: DateInput<'a>) -> Self {
        value.view()
    }
}
