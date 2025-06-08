use super::Shift;
use iced::widget;
use time::Date;

#[derive(Debug, Clone)]
pub enum Action {
    ChangeStart(super::date_input::Action),
    ChangeEnd(super::date_input::Action),
    Shift { shift: super::Shift, positive: bool },
    ToggleDropdown,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    start: super::date_input::State,
    end: super::date_input::State,
    drop_down: bool,
}

impl State {
    pub fn new(timespan: Option<(Option<Date>, Option<Date>)>) -> Self {
        Self {
            start: super::date_input::State::new(if let Some(t) = timespan { t.0 } else { None }),
            end: super::date_input::State::new(if let Some(t) = timespan { t.1 } else { None }),
            drop_down: false,
        }
    }

    pub fn timespan(&self) -> (Option<time::Date>, Option<Date>) {
        (self.start.date(), self.end.date())
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::ChangeStart(start) => {
                self.start.perform(start);
            }
            Action::ChangeEnd(end) => {
                self.end.perform(end);
            }
            Action::Shift { shift, positive } => {
                if let Some(before) = self.start.date() {
                    self.start = super::date_input::State::new(Some(super::apply_date_shift(
                        before, shift, positive,
                    )));
                }
                if let Some(before) = self.end.date() {
                    self.end = super::date_input::State::new(Some(super::apply_date_shift(
                        before, shift, positive,
                    )));
                }
            }
            Action::ToggleDropdown => {
                self.drop_down = !self.drop_down;
            }
        }
    }
}

pub struct DateSpanInput<'a> {
    state: &'a State,
}

impl<'a> DateSpanInput<'a> {
    pub fn new(state: &'a State) -> Self {
        DateSpanInput { state }
    }

    pub fn view(self) -> iced::Element<'a, Action> {
        crate::drop_down(
            crate::spal_row![
                super::date_input::date_input(&self.state.start, "Start", false)
                    .view()
                    .map(Action::ChangeStart),
                " - ",
                super::date_input::date_input(&self.state.end, "End", false)
                    .view()
                    .map(Action::ChangeEnd),
                widget::button(icons::pencil_fill()).on_press_maybe(
                    if self.state.start.date().is_some() && self.state.end.date().is_some() {
                        Some(Action::ToggleDropdown)
                    } else {
                        None
                    }
                )
            ]
            .align_y(iced::Alignment::Center),
            crate::spal_column![
                crate::spal_row![
                    widget::button("-").on_press(Action::Shift {
                        shift: Shift::Duration(time::Duration::DAY),
                        positive: false
                    }),
                    "Day",
                    widget::button("+").on_press(Action::Shift {
                        shift: Shift::Duration(time::Duration::DAY),
                        positive: true
                    }),
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::Shift {
                        shift: Shift::Duration(time::Duration::WEEK),
                        positive: false
                    }),
                    "Week",
                    widget::button("+").on_press(Action::Shift {
                        shift: Shift::Duration(time::Duration::WEEK),
                        positive: true
                    }),
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::Shift {
                        shift: Shift::Month,
                        positive: false
                    }),
                    "Month",
                    widget::button("+").on_press(Action::Shift {
                        shift: Shift::Month,
                        positive: true
                    }),
                ],
                crate::spal_row![
                    widget::button("-").on_press(Action::Shift {
                        shift: Shift::Year,
                        positive: false
                    }),
                    "Year",
                    widget::button("+").on_press(Action::Shift {
                        shift: Shift::Year,
                        positive: true
                    }),
                ]
            ],
            self.state.drop_down,
        )
        .on_dismiss(Action::ToggleDropdown)
        .into()
    }
}

pub fn date_span_input(state: &State) -> DateSpanInput<'_> {
    DateSpanInput { state }
}

impl<'a> From<DateSpanInput<'a>> for iced::Element<'a, Action> {
    fn from(value: DateSpanInput<'a>) -> Self {
        value.view()
    }
}
