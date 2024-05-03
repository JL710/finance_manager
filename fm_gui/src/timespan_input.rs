use super::utils;
use iced::widget;

pub struct TimespanInput<Message, PM: Fn(fm_core::Timespan) -> Message> {
    produce_message: PM,
}

impl<'a, Message: 'a, PM: Fn(fm_core::Timespan) -> Message + 'a> TimespanInput<Message, PM> {
    pub fn new(produce_message: PM) -> Self {
        Self { produce_message }
    }

    pub fn into_element(self) -> iced::Element<'a, Message> {
        widget::component(self)
    }
}

#[derive(Debug, Clone)]
pub enum TimespanInputMsg {
    SetStart(String),
    SetEnd(String),
}

#[derive(Debug, Clone, Default)]
pub struct TimespanInputState {
    start: String,
    end: String,
}

impl<Message, PM: Fn(fm_core::Timespan) -> Message> widget::Component<Message>
    for TimespanInput<Message, PM>
{
    type Event = TimespanInputMsg;
    type State = TimespanInputState;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            TimespanInputMsg::SetStart(start) => {
                state.start = start;
            }
            TimespanInputMsg::SetEnd(end) => {
                state.end = end;
            }
        }
        let start = match utils::parse_to_datetime(&state.start) {
            Ok(x) => Some(x),
            Err(_) => {
                if !state.start.is_empty() {
                    return None;
                }
                None
            }
        };
        let end = match utils::parse_to_datetime(&state.end) {
            Ok(x) => Some(x),
            Err(_) => {
                if !state.end.is_empty() {
                    return None;
                }
                None
            }
        };
        Some((self.produce_message)((start, end)))
    }

    fn view(
        &self,
        state: &Self::State,
    ) -> iced::Element<'_, Self::Event> {
        widget::row![
            widget::text_input("Start", &state.start).on_input(TimespanInputMsg::SetStart),
            widget::text(" - "),
            widget::text_input("End", &state.end).on_input(TimespanInputMsg::SetEnd),
        ]
        .into()
    }
}
