#[derive(Default, Debug, Clone)]
pub struct State {
    value: String,
    default_value: Option<fm_core::DateTime>,
}

impl State {
    pub fn new(value: Option<fm_core::DateTime>) -> Self {
        Self {
            value: if let Some(date) = value {
                date.format(&time::format_description::parse("[day].[month].[year]").unwrap())
                    .unwrap()
            } else {
                String::new()
            },
            default_value: None,
        }
    }

    pub fn new_with_raw(value: String) -> Self {
        Self {
            value,
            default_value: None,
        }
    }

    pub fn default_value(&mut self, value: Option<fm_core::DateTime>) {
        self.default_value = value;
    }

    /// Will return the user input as datetime if possible or default datetime if given.
    pub fn date(&self) -> Option<fm_core::DateTime> {
        match super::parse_to_datetime(&self.value) {
            Ok(value) => Some(value),
            _ => match self.default_value {
                Some(value) => Some(value),
                _ => None,
            },
        }
    }

    pub fn raw_input(&self) -> &str {
        &self.value
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::InputChanged(new_value) => self.value = new_value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    InputChanged(String),
}

pub struct DateInput<'a> {
    required: bool,
    state: &'a State,
    placeholder: String,
}

impl<'a> DateInput<'a> {
    pub fn view(self) -> iced::Element<'a, Action> {
        iced::widget::text_input("Date", self.state.raw_input())
            .on_input(Action::InputChanged)
            .style(move |theme: &iced::Theme, status| {
                let mut original = iced::widget::text_input::default(theme, status);

                if (self.state.raw_input().is_empty() && self.required)
                    || (!self.state.raw_input().is_empty() && self.state.date().is_none())
                {
                    original.border.color = theme.palette().danger;
                } else if !self.state.raw_input().is_empty() && self.state.date().is_some() {
                    original.border.color = theme.palette().success;
                }
                original
            })
            .into()
    }
}

pub fn date_input<'a>(state: &'a State, placeholder: &'a str, required: bool) -> DateInput<'a> {
    DateInput {
        state,
        required,
        placeholder: placeholder.to_owned(),
    }
}

impl<'a> From<DateInput<'a>> for iced::Element<'a, Action> {
    fn from(value: DateInput<'a>) -> Self {
        value.view()
    }
}
