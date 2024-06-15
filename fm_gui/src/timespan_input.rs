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

    fn view(&self, state: &Self::State) -> iced::Element<'_, Self::Event> {
        fn incorrect_style(
            theme: &iced::Theme,
            status: widget::text_input::Status,
        ) -> widget::text_input::Style {
            let mut style = widget::text_input::default(theme, status);
            style.border = style.border.with_color(theme.palette().danger);
            style
        }

        let start_correct =
            utils::parse_to_datetime(&state.start).is_ok() || state.start.is_empty();
        let end_correct = utils::parse_to_datetime(&state.end).is_ok() || state.end.is_empty();
        widget::row![
            widget::text_input("Start", &state.start)
                .style(if start_correct {
                    widget::text_input::default
                } else {
                    incorrect_style
                })
                .on_input(TimespanInputMsg::SetStart),
            widget::text(" - "),
            widget::text_input("End", &state.end)
                .style(if end_correct {
                    widget::text_input::default
                } else {
                    incorrect_style
                })
                .on_input(TimespanInputMsg::SetEnd),
        ]
        .align_items(iced::Alignment::Center)
        .width(iced::Length::Fixed(300.0))
        .into()
    }
}
