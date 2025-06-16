use iced::widget;

type ValidationFn = Box<dyn Fn(&String) -> Option<String>>;

pub struct ValidationTextInput {
    content: String,
    validation: ValidationFn,
    /// If this is some there the current content is invalid else it is valid
    error_message: Option<String>,
    input_changed: bool,
    required: bool,
}

impl Default for ValidationTextInput {
    fn default() -> Self {
        Self::new(String::default())
    }
}

impl std::fmt::Debug for ValidationTextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidationTextInput {{content: {}, error_message: {}, input_changed: {}, required: {} }}",
            self.content,
            self.error_message
                .as_ref()
                .map_or(String::new(), |x| x.to_owned()),
            self.input_changed,
            self.required
        )
    }
}

impl ValidationTextInput {
    pub fn new(content: String) -> Self {
        Self {
            error_message: None,
            content,
            validation: Box::new(|_| None),
            input_changed: false,
            required: false,
        }
    }

    pub fn validation(mut self, validation: impl Fn(&String) -> Option<String> + 'static) -> Self {
        self.validation = Box::new(validation);
        self.validate();
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self.validate();
        self
    }

    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }

    pub fn error_message(&self) -> Option<&String> {
        self.error_message.as_ref()
    }

    pub fn edit_content(&mut self, content: String) {
        self.content = content;
        self.validate();
        self.input_changed = true;
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.validate();
        self.input_changed = false;
    }

    fn validate(&mut self) {
        if self.required && self.content.is_empty() {
            self.error_message = Some("required input is empty".to_string());
        } else {
            self.error_message = (self.validation)(&self.content);
        }
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
        widget::column![
            widget::row![
                widget::text_input(placeholder, &self.content)
                    .on_input_maybe(on_input)
                    .style(if !self.input_changed {
                        widget::text_input::default
                    } else if self.is_valid() {
                        style::text_input_success
                    } else {
                        style::text_input_danger
                    })
            ]
            .push_maybe(if self.required { Some("*") } else { None })
        ]
        .push_maybe(
            if let Some(error_message) = &self.error_message
                && self.input_changed
            {
                Some(widget::container(error_message.as_str()).padding(3).style(
                    |theme: &iced::Theme| {
                        let mut style = widget::container::background(theme.palette().danger);
                        style.border = style.border.rounded(3);
                        style
                    },
                ))
            } else {
                None
            },
        )
        .into()
    }
}
