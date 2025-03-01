use iced::widget;

#[derive(Debug, Clone)]
pub enum Message {
    OnInput(String),
}

#[derive(Debug)]
pub struct State {
    value: String,
}

impl State {
    pub fn new(default: Option<time::Time>) -> Self {
        Self {
            value: if let Some(t) = default {
                super::to_time_string(t)
            } else {
                String::default()
            },
        }
    }

    pub fn perform(&mut self, message: Message) {
        match message {
            Message::OnInput(input) => self.value = input,
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
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn view(self) -> iced::Element<'a, Message> {
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
            .into()
    }
}

pub fn time_input<'a>(state: &'a State, required: bool) -> TimeInput<'a> {
    TimeInput { state, required }
}
