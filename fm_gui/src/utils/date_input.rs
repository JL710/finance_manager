pub struct DateInput<Message: Clone> {
    produce_message: Box<dyn Fn(Option<fm_core::DateTime>) -> Message>,
    default_value: Option<fm_core::DateTime>,
    required: bool,
}

impl<'a, Message: Clone + 'a> DateInput<Message> {
    pub fn new(produce_message: impl Fn(Option<fm_core::DateTime>) -> Message + 'static) -> Self {
        Self {
            produce_message: Box::new(produce_message),
            default_value: None,
            required: false,
        }
    }

    pub fn default_value(mut self, default_value: Option<fm_core::DateTime>) -> Self {
        self.default_value = default_value;
        self
    }

    /// Marks the input as red if it is empty
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    fn get_value_as_string(&self, state: &State) -> String {
        match state.input {
            Some(ref x) => x.clone(),
            None => match self.default_value {
                Some(x) => x.format("%d.%m.%Y").to_string(),
                None => String::new(),
            },
        }
    }
}

impl<'a, Message: Clone + 'a> From<DateInput<Message>> for iced::Element<'a, Message> {
    fn from(date_input: DateInput<Message>) -> iced::Element<'a, Message> {
        iced::widget::component(date_input)
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    input: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Event {
    InputChanged(String),
}

impl<Message: Clone> iced::widget::Component<Message> for DateInput<Message> {
    type Event = Event;
    type State = State;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::InputChanged(input) => {
                state.input = Some(input);
                if let Ok(x) = super::parse_to_datetime(state.input.as_ref().unwrap()) {
                    Some((self.produce_message)(Some(x)))
                } else {
                    Some((self.produce_message)(None))
                }
            }
        }
    }

    fn view(&self, state: &Self::State) -> iced::Element<'_, Self::Event> {
        let value = self.get_value_as_string(state);

        let date_is_valid_parsed = super::parse_to_datetime(&value).is_ok();
        let input_is_empty = value.is_empty();

        iced::widget::text_input("Date", &value)
            .on_input(Event::InputChanged)
            .style(move |theme: &iced::Theme, status| {
                let mut original = iced::widget::text_input::default(theme, status);

                if (input_is_empty && self.required) || (!input_is_empty && !date_is_valid_parsed) {
                    original.border.color = theme.palette().danger;
                } else if !input_is_empty && date_is_valid_parsed {
                    original.border.color = theme.palette().success;
                }
                original
            })
            .into()
    }
}
