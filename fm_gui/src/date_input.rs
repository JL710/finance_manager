pub struct DateInput<Message: Clone> {
    produce_message: Box<dyn Fn(Option<fm_core::DateTime>) -> Message>,
}

impl<'a, Message: Clone + 'a> DateInput<Message> {
    pub fn new(produce_message: impl Fn(Option<fm_core::DateTime>) -> Message + 'static) -> Self {
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

impl<Message: Clone> iced::widget::Component<Message> for DateInput<Message> {
    type Event = Event;
    type State = State;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::InputChanged(input) => {
                state.input = input;
                if let Ok(x) = super::utils::parse_to_datetime(&state.input) {
                    return Some((self.produce_message)(Some(x)));
                }
                if state.input.is_empty() {
                    return Some((self.produce_message)(None));
                }
            }
        }
        None
    }

    fn view(&self, state: &Self::State) -> iced::Element<'_, Self::Event> {
        let date_is_correct =
            !&state.input.is_empty() && super::utils::parse_to_datetime(&state.input).is_err();

        iced::widget::text_input("Date", &state.input)
            .on_input(Event::InputChanged)
            .style(move |theme: &iced::Theme, status| {
                let mut original = iced::widget::text_input::default(theme, status);
                if date_is_correct {
                    original.border.color = theme.palette().danger;
                }
                original
            })
            .into()
    }
}
