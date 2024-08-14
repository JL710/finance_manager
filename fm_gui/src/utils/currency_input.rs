pub struct CurrencyInput<Message: Clone> {
    value: Option<fm_core::Currency>,
    produce_message: Box<dyn Fn(Option<fm_core::Currency>) -> Message>,
}

impl<'a, Message: Clone + 'a> CurrencyInput<Message> {
    pub fn new(
        value: Option<fm_core::Currency>,
        produce_message: impl Fn(Option<fm_core::Currency>) -> Message + 'static,
    ) -> Self {
        Self {
            produce_message: Box::new(produce_message),
            value,
        }
    }

    pub fn into_element(self) -> iced::Element<'a, Message> {
        iced::widget::component(self)
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

impl<Message: Clone> iced::widget::Component<Message> for CurrencyInput<Message> {
    type Event = Event;
    type State = State;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::InputChanged(input) => {
                state.input = Some(input);
                if let Some(x) = super::parse_number(state.input.as_ref().unwrap()) {
                    return Some((self.produce_message)(Some(fm_core::Currency::from(x))));
                }
                Some((self.produce_message)(None))
            }
        }
    }

    fn view(&self, state: &Self::State) -> iced::Element<'_, Self::Event> {
        let value = if let Some(v) = &state.input {
            v.clone()
        } else if let Some(v) = &self.value {
            v.to_num_string()
        } else {
            String::new()
        };
        let correct = !&value.is_empty() && super::parse_number(&value).is_none();

        iced::widget::row![
            iced::widget::text_input("Value", &value)
                .on_input(Event::InputChanged)
                .style(move |theme: &iced::Theme, status| {
                    let mut original = iced::widget::text_input::default(theme, status);
                    if correct {
                        original.border.color = theme.palette().danger;
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
