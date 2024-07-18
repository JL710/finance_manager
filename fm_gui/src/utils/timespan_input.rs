use iced::widget;

pub struct TimespanInput<Message, PM: Fn(fm_core::Timespan) -> Message> {
    produce_message: PM,
    default_timespan: Option<fm_core::Timespan>,
}

impl<'a, Message: 'a, PM: Fn(fm_core::Timespan) -> Message + 'a> TimespanInput<Message, PM> {
    pub fn new(produce_message: PM, default_timespan: Option<fm_core::Timespan>) -> Self {
        Self {
            produce_message,
            default_timespan,
        }
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
    start: Option<String>,
    end: Option<String>,
}

impl<Message, PM: Fn(fm_core::Timespan) -> Message> widget::Component<Message>
    for TimespanInput<Message, PM>
{
    type Event = TimespanInputMsg;
    type State = TimespanInputState;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            TimespanInputMsg::SetStart(start) => {
                state.start = Some(start);
            }
            TimespanInputMsg::SetEnd(end) => {
                state.end = Some(end);
            }
        }

        // overwrite empty start and end with default values if not already set
        if state.start.is_none() {
            if let Some(default_timespan) = self.default_timespan.as_ref() {
                state.start = Some(
                    default_timespan
                        .0
                        .map_or(String::new(), |x| x.format("%d.%m.%Y").to_string()),
                );
            } else {
                state.start = Some(String::new());
            }
        }
        if state.end.is_none() {
            if let Some(default_timespan) = self.default_timespan.as_ref() {
                state.end = Some(
                    default_timespan
                        .1
                        .map_or(String::new(), |x| x.format("%d.%m.%Y").to_string()),
                );
            } else {
                state.end = Some(String::new());
            }
        }

        let start = match super::parse_to_datetime(state.start.as_ref().unwrap()) {
            Ok(x) => Some(x),
            Err(_) => {
                if !state.start.as_ref().unwrap().is_empty() {
                    return None;
                }
                None
            }
        };
        let end = match super::parse_to_datetime(state.end.as_ref().unwrap()) {
            Ok(x) => Some(x),
            Err(_) => {
                if !state.end.as_ref().unwrap().is_empty() {
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
            style.border = style.border.color(theme.palette().danger);
            style
        }

        let start_correct = super::parse_to_datetime(&state.start.clone().unwrap_or_default())
            .is_ok()
            || state.start.clone().unwrap_or_default().is_empty();
        let end_correct = super::parse_to_datetime(&state.end.clone().unwrap_or_default()).is_ok()
            || state.end.clone().unwrap_or_default().is_empty();
        widget::row![
            widget::text_input(
                "Start",
                &state
                    .start
                    .clone()
                    .unwrap_or(self.default_timespan.map_or(String::new(), |x| {
                        x.0.map_or(String::new(), |y| y.format("%d.%m.%Y").to_string())
                    }))
            )
            .style(if start_correct {
                widget::text_input::default
            } else {
                incorrect_style
            })
            .on_input(TimespanInputMsg::SetStart),
            widget::text(" - "),
            widget::text_input(
                "End",
                &state
                    .end
                    .clone()
                    .unwrap_or(self.default_timespan.map_or(String::new(), |x| {
                        x.1.map_or(String::new(), |y| y.format("%d.%m.%Y").to_string())
                    }))
            )
            .style(if end_correct {
                widget::text_input::default
            } else {
                incorrect_style
            })
            .on_input(TimespanInputMsg::SetEnd),
        ]
        .align_y(iced::Alignment::Center)
        .width(iced::Length::Fixed(300.0))
        .into()
    }
}
