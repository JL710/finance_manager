use super::{date_input, time_input};

#[derive(Debug, Clone)]
pub enum Action {
    DateInput(date_input::Action),
    TimeInput(time_input::Message),
}

#[derive(Debug, Default)]
pub struct State {
    date_input: date_input::State,
    time_input: time_input::State,
}

impl State {
    pub fn new(default: Option<time::PrimitiveDateTime>) -> Self {
        Self {
            date_input: date_input::State::new(default.map(|x| x.date())),
            time_input: time_input::State::new(default.map(|default| default.time())),
        }
    }

    pub fn perform(&mut self, message: Action) {
        match message {
            Action::DateInput(input) => self.date_input.perform(input),
            Action::TimeInput(input) => self.time_input.perform(input),
        }
    }

    pub fn datetime(&self) -> Option<time::PrimitiveDateTime> {
        if let Some(date) = self.date_input.date()
            && let Some(time) = self.time_input.time()
        {
            return Some(time::PrimitiveDateTime::new(date, time));
        }
        None
    }
}

pub struct DateTimeInput<'a> {
    state: &'a State,
    required: bool,
}

impl<'a> DateTimeInput<'a> {
    pub fn new(state: &'a State, required: bool) -> Self {
        Self { state, required }
    }

    pub fn view(self) -> iced::Element<'a, Action> {
        crate::spal_row![
            date_input::date_input(
                &self.state.date_input,
                "Date",
                self.required || self.state.time_input.anything_entered()
            )
            .view()
            .map(Action::DateInput),
            time_input::time_input(
                &self.state.time_input,
                self.required || self.state.date_input.anything_entered()
            )
            .view()
            .map(Action::TimeInput)
        ]
        .into()
    }
}

pub fn date_time_input(state: &State, required: bool) -> DateTimeInput<'_> {
    DateTimeInput::new(state, required)
}
