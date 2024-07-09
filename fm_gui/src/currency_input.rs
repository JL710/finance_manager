pub struct CurrencyInput<Message: Clone> {
    produce_message: Box<dyn Fn(fm_core::Currency) -> Message>,
}

impl<'a, Message: Clone + 'a> CurrencyInput<Message> {
    pub fn new(produce_message: impl Fn(fm_core::Currency) -> Message + 'static) -> Self {
        Self {
            produce_message: Box::new(produce_message),
        }
    }

    pub fn into_element(self) -> iced::Element<'a, Message> {
        iced::widget::component(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    input: String,
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
                state.input = input;
                if let Ok(x) = state.input.parse::<f64>() {
                    return Some((self.produce_message)(fm_core::Currency::Eur(x)));
                }
            }
        }
        None
    }

    fn view(&self, state: &Self::State) -> iced::Element<'_, Self::Event> {
        let date_is_correct = !&state.input.is_empty() && state.input.parse::<f64>().is_err();

        iced::widget::row![
            iced::widget::text_input("Value", &state.input)
                .on_input(Event::InputChanged)
                .style(move |theme: &iced::Theme, status| {
                    let mut original = iced::widget::text_input::default(theme, status);
                    if date_is_correct {
                        original.border.color = theme.palette().danger;
                    }
                    original
                }),
            "€",
        ]
        .align_items(iced::Alignment::Center)
        .spacing(10)
        .into()
    }
}
