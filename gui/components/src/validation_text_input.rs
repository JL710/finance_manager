use iced::widget;

pub struct ValidationTextInput {
    content: String,
    validation: Box<dyn Fn(&String) -> Option<String>>,
    /// If this is some there the current content is invalid else it is valid
    error_message: Option<String>,
    input_changed: bool,
}

impl std::fmt::Debug for ValidationTextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidationTextInput {{content: {}, error_message: {}, input_changed: {} }}",
            &self.content,
            self.error_message
                .as_ref()
                .map_or(String::new(), |x| x.to_owned()),
            &self.input_changed
        )
    }
}

impl ValidationTextInput {
    pub fn new(content: String, validation: impl Fn(&String) -> Option<String> + 'static) -> Self {
        Self {
            error_message: validation(&content),
            content,
            validation: Box::new(validation),
            input_changed: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.error_message = (self.validation)(&self.content);
        self.input_changed = true;
    }

    /// Sets if an input was already entered
    pub fn input_changed(&mut self, input_changed: bool) {
        self.input_changed = input_changed;
    }

    pub fn value(&self) -> &String {
        &self.content
    }

    pub fn view<'a, Message: Clone + 'a>(
        &'a self,
        placeholder: &str,
        on_input: Option<impl Fn(String) -> Message + 'a>,
    ) -> iced::Element<'a, Message> {
        let mut col = widget::column![
            widget::text_input(placeholder, &self.content)
                .on_input_maybe(on_input)
                .style(if !self.input_changed {
                    widget::text_input::default
                } else if self.is_valid() {
                    style::text_input_success
                } else {
                    style::text_input_danger
                })
        ];

        if let Some(error_message) = &self.error_message
            && self.input_changed
        {
            col = col.push(widget::container(error_message.as_str()).padding(3).style(
                |theme: &iced::Theme| {
                    let mut style = widget::container::background(theme.palette().danger);
                    style.border = style.border.rounded(3);
                    style
                },
            ))
        }

        col.into()
    }
}
