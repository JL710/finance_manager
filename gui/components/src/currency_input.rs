#[derive(Clone, Debug)]
pub enum Action {
    Input(String),
}

#[derive(Debug)]
pub struct CurrencyInput {
    value: crate::ValidationTextInput,
}

impl Default for CurrencyInput {
    fn default() -> Self {
        let mut new = Self::new(fm_core::Currency::default(), true);
        new.clear();
        new
    }
}

impl CurrencyInput {
    pub fn new(value: fm_core::Currency, required: bool) -> Self {
        Self {
            value: crate::ValidationTextInput::new(value.to_num_string(), move |content| {
                if !required && content.is_empty() {
                    None
                } else if content.is_empty() {
                    Some("empty input".to_string())
                } else if let None = super::parse_number(content) {
                    Some("invalid number".to_string())
                } else {
                    None
                }
            }),
        }
    }

    pub fn clear(&mut self) {
        self.value.set_content(String::new());
        self.value.input_changed(false);
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Input(input) => {
                self.value.set_content(input);
            }
        }
    }

    pub fn currency(&self) -> Option<fm_core::Currency> {
        super::parse_number(self.value.value()).map(fm_core::Currency::from)
    }

    pub fn set_value(&mut self, new_value: fm_core::Currency) {
        self.value.set_content(new_value.to_num_string());
        self.value.input_changed(false);
    }

    pub fn view(&self) -> iced::Element<Action> {
        super::spal_row![self.value.view("Value", Some(Action::Input)), "€",]
            .align_y(iced::Alignment::Center)
            .into()
    }
}
