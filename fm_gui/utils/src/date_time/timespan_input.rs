use iced::widget;

#[derive(Debug, Clone)]
pub enum Action {
    ChangeStart(super::date_input::Action),
    ChangeEnd(super::date_input::Action),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    start: super::date_input::State,
    end: super::date_input::State,
}

impl State {
    pub fn new(timespan: Option<fm_core::Timespan>) -> Self {
        Self {
            start: super::date_input::State::new(if let Some(t) = timespan { t.0 } else { None }),
            end: super::date_input::State::new(if let Some(t) = timespan { t.1 } else { None }),
        }
    }

    pub fn timespan(&self) -> fm_core::Timespan {
        (
            self.start
                .date()
                .map(|x| x.replace_time(time::Time::from_hms(0, 0, 0).unwrap())),
            self.end
                .date()
                .map(|x| x.replace_time(time::Time::from_hms(23, 59, 59).unwrap())),
        )
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::ChangeStart(start) => {
                self.start.perform(start);
            }
            Action::ChangeEnd(end) => {
                self.end.perform(end);
            }
        }
    }
}

pub struct TimespanInput<'a> {
    state: &'a State,
}

impl<'a> TimespanInput<'a> {
    pub fn new(state: &'a State) -> Self {
        TimespanInput { state }
    }

    pub fn view(self) -> iced::Element<'a, Action> {
        widget::row![
            super::date_input::date_input(&self.state.start, "Start", false)
                .view()
                .map(Action::ChangeStart),
            widget::text(" - "),
            super::date_input::date_input(&self.state.end, "End", false)
                .view()
                .map(Action::ChangeEnd),
        ]
        .align_y(iced::Alignment::Center)
        .width(iced::Length::Fixed(300.0))
        .into()
    }
}

pub fn timespan_input(state: &State) -> TimespanInput<'_> {
    TimespanInput { state }
}

impl<'a> From<TimespanInput<'a>> for iced::Element<'a, Action> {
    fn from(value: TimespanInput<'a>) -> Self {
        value.view()
    }
}
