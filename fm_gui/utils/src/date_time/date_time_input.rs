use super::{date_input, time_input};

#[derive(Debug, Clone)]
pub enum Action {
    DateInput(date_input::Action),
    TimeInput(time_input::Message),
}

#[derive(Debug)]
pub struct State {
    date_input: date_input::State,
    time_input: time_input::State,
}

impl Default for State {
    fn default() -> Self {
        Self {
            date_input: date_input::State::default(),
            time_input: time_input::State::new(None),
        }
    }
}

impl State {
    pub fn new(default: Option<time::OffsetDateTime>) -> Self {
        Self {
            date_input: date_input::State::new(default),
            time_input: time_input::State::new(if let Some(default) = default {
                Some(default.time())
            } else {
                Some(time::Time::from_hms(12, 0, 0).unwrap())
            }),
        }
    }

    pub fn perform(&mut self, message: Action) {
        match message {
            Action::DateInput(input) => self.date_input.perform(input),
            Action::TimeInput(input) => self.time_input.perform(input),
        }
    }

    pub fn datetime(&self) -> Option<time::OffsetDateTime> {
        if let Some(date) = self.date_input.date() {
            if let Some(time) = self.time_input.time() {
                return Some(date.replace_time(time));
            }
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

pub fn date_time_input<'a>(state: &'a State, required: bool) -> DateTimeInput<'a> {
    DateTimeInput::new(state, required)
}
