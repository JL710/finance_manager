use iced::widget;

pub enum Action {
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    KeyInput(usize, String),
    ValueInput(usize, widget::text_editor::Action),
    AddPair,
    DeletePair(usize),
}

#[derive(Default, Debug)]
pub struct KeyValueEditor {
    inputs: Vec<(String, widget::text_editor::Content)>,
}

impl KeyValueEditor {
    pub fn new(key_value_pairs: Vec<(String, String)>) -> Self {
        Self {
            inputs: key_value_pairs
                .into_iter()
                .map(|pair| (pair.0, widget::text_editor::Content::with_text(&pair.1)))
                .collect(),
        }
    }

    pub fn pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::with_capacity(self.inputs.len());
        for pair in &self.inputs {
            pairs.push((pair.0.clone(), pair.1.text()));
        }
        pairs
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::KeyInput(index, new_value) => {
                if let Some(pair) = self.inputs.get_mut(index) {
                    pair.0 = new_value;
                }
            }
            Message::ValueInput(index, action) => {
                if let Some(pair) = self.inputs.get_mut(index) {
                    pair.1.perform(action);
                }
            }
            Message::AddPair => {
                self.inputs
                    .push((String::new(), widget::text_editor::Content::default()));
            }
            Message::DeletePair(index) => {
                if index > self.inputs.len() - 1 {
                    self.inputs.remove(index);
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let mut column = super::spaced_column!();
        for (index, pair) in self.inputs.iter().enumerate() {
            column = column.push(super::spal_row![
                widget::text_input("Key", &pair.0).on_input(move |x| Message::KeyInput(index, x)),
                widget::text_editor(&pair.1)
                    .on_action(move |action| Message::ValueInput(index, action)),
                super::button::delete(Some(Message::DeletePair(index)))
            ])
        }

        super::spaced_column![
            widget::scrollable(column),
            super::button::new("Add", Some(Message::AddPair))
        ]
        .into()
    }
}

impl From<std::collections::HashMap<String, String>> for KeyValueEditor {
    fn from(value: std::collections::HashMap<String, String>) -> Self {
        Self {
            inputs: value
                .into_iter()
                .map(|pair| (pair.0, widget::text_editor::Content::with_text(&pair.1)))
                .collect(),
        }
    }
}

impl From<KeyValueEditor> for std::collections::HashMap<String, String> {
    fn from(value: KeyValueEditor) -> Self {
        std::collections::HashMap::from_iter(value.pairs().into_iter())
    }
}
