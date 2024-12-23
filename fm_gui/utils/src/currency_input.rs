#[derive(Clone, Debug)]
pub enum Action {
    Input(String),
}

#[derive(Default, Clone, Debug)]
pub struct State {
    value: String,
}

impl State {
    pub fn new(value: fm_core::Currency) -> Self {
        Self {
            value: value.to_num_string(),
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Input(input) => {
                self.value = input;
            }
        }
    }

    pub fn currency(&self) -> Option<fm_core::Currency> {
        if let Some(x) = super::parse_number(&self.value) {
            Some(fm_core::Currency::from(x))
        } else {
            None
        }
    }
}

pub struct CurrencyInput<'a> {
    required: bool,
    state: &'a State,
}

impl<'a> CurrencyInput<'a> {
    pub fn new(state: &'a State, required: bool) -> Self {
        Self { state, required }
    }

    pub fn view(self) -> iced::Element<'a, Action> {
        let wrong = (!self.state.value.is_empty()
            && super::parse_number(&self.state.value).is_none())
            || (self.required && self.state.value.is_empty());

        iced::widget::row![
            iced::widget::text_input("Value", &self.state.value)
                .on_input(Action::Input)
                .style(move |theme: &iced::Theme, status| {
                    let mut original = iced::widget::text_input::default(theme, status);
                    if wrong {
                        original.border.color = theme.palette().danger;
                    } else if self.required {
                        original.border.color = theme.palette().success;
                    }
                    original
                }),
            "€",
        ]
        .align_y(iced::Alignment::Center)
        .spacing(10)
        .into()
    }
}

pub fn currency_input<'a>(state: &'a State, required: bool) -> CurrencyInput<'a> {
    CurrencyInput::new(state, required)
}
