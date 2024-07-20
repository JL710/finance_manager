use iced::widget;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Action {
    LineChange(usize, String),
    Submit,
    Edit,
    /// Add a new line after this index
    AddLine(usize),
    DeleteLine(usize),
}

#[derive(Debug, Clone, Default)]
pub struct State {
    content: String,
    lines: Option<Vec<String>>,
}

impl State {
    pub fn new(content: String) -> Self {
        Self {
            content,
            lines: None,
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::LineChange(index, line) => {
                if let Some(lines) = &mut self.lines {
                    lines[index] = line;
                }
            }
            Action::Submit => {
                self.content = self
                    .lines
                    .as_ref()
                    .unwrap()
                    .iter()
                    .fold(String::new(), |acc, x| acc + x + "\n");
                self.lines = None;
            }
            Action::Edit => {
                self.lines = Some(self.content.lines().map(|x| x.to_string()).collect());
                if let Some(lines) = &mut self.lines {
                    if lines.is_empty() {
                        lines.push(String::new());
                    }
                }
            }
            Action::DeleteLine(index) => {
                if let Some(lines) = &mut self.lines {
                    if lines.len() > 1 {
                        lines.remove(index);
                    }
                }
            }
            Action::AddLine(index) => {
                if let Some(lines) = &mut self.lines {
                    lines.insert(index + 1, String::new());
                }
            }
        }
    }

    pub fn get_content(&self) -> &String {
        &self.content
    }
}

pub struct MultilineTextInput<'a, Message: Clone + std::fmt::Debug> {
    state: State,
    on_action: Option<Arc<Box<dyn Fn(Action) -> Message + 'a>>>,
}

impl<'a, Message: Clone + std::fmt::Debug> MultilineTextInput<'a, Message>
where
    Self: 'a,
{
    pub fn new(state: State) -> Self {
        Self {
            state,
            on_action: None,
        }
    }

    pub fn on_action(mut self, on_action: impl Fn(Action) -> Message + 'a) -> Self {
        self.on_action = Some(Arc::new(Box::new(on_action)));
        self
    }

    pub fn view(&self) -> iced::Element<'a, Message> {
        if let Some(lines) = &self.state.lines {
            let mut content = widget::column![];
            for (index, line) in lines.iter().enumerate() {
                content = content.push(widget::row![
                    if let Some(on_action) = &self.on_action {
                        let cloned_on_action = on_action.clone();
                        widget::text_input("", line)
                            .on_input(move |x| (cloned_on_action)(Action::LineChange(index, x)))
                            .on_submit((on_action)(Action::AddLine(index)))
                    } else {
                        widget::text_input("", line)
                    },
                    widget::button("Delete").on_press_maybe(
                        self.on_action
                            .as_ref()
                            .map(|on_action| (on_action)(Action::DeleteLine(index)))
                    )
                ]);
            }
            return widget::column![
                content,
                widget::button("Submit").on_press_maybe(
                    self.on_action
                        .as_ref()
                        .map(|on_action| (on_action)(Action::Submit))
                ),
            ]
            .spacing(10)
            .into();
        }
        widget::row![
            widget::text(self.state.content.clone()),
            widget::button("Edit").on_press_maybe(
                self.on_action
                    .as_ref()
                    .map(|on_action| (on_action)(Action::Edit))
            )
        ]
        .spacing(10)
        .into()
    }
}
