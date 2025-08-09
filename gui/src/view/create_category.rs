use anyhow::Context;

pub enum Action {
    None,
    CategoryCreated(fm_core::Id),
    Task(iced::Task<Message>),
    Cancel,
    CancelWithId(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    NameInput(String),
    CategoryCreated(fm_core::Id),
    Initialize(fm_core::Category),
    Cancel,
    Reload { exists: bool },
}

#[derive(Debug)]
pub struct View {
    id: Option<fm_core::Id>,
    name: components::ValidationTextInput,
    submitted: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            id: None,
            name: components::ValidationTextInput::default().required(true),
            submitted: false,
        }
    }
}

impl View {
    pub fn new(id: Option<fm_core::Id>, name: String) -> Self {
        Self {
            id,
            name: components::ValidationTextInput::new(name).required(true),
            submitted: false,
        }
    }

    pub fn reload(
        &self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> iced::Task<Message> {
        if let Some(id) = self.id {
            error::failing_task(async move {
                Ok(Message::Reload {
                    exists: finance_controller.get_category(id).await?.is_some(),
                })
            })
        } else {
            iced::Task::none()
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(None, String::new()),
            error::failing_task(async move {
                let category = finance_controller
                    .get_category(id)
                    .await?
                    .context(format!("Could not find category {id}"))?;
                Ok(Message::Initialize(category))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Reload { exists } => {
                if !exists {
                    self.id = None;
                }
                Action::None
            }
            Message::Cancel => {
                if let Some(id) = self.id {
                    Action::CancelWithId(id)
                } else {
                    Action::Cancel
                }
            }
            Message::CategoryCreated(id) => Action::CategoryCreated(id),
            Message::Initialize(category) => {
                self.id = Some(category.id);
                self.name.set_content(category.name);
                Action::None
            }
            Message::NameInput(content) => {
                self.name.edit_content(content);
                Action::None
            }
            Message::Submit => {
                self.submitted = true;
                let id = self.id;
                let name = self.name.value().clone();
                Action::Task(error::failing_task(async move {
                    if let Some(id) = id {
                        Ok(Message::CategoryCreated(
                            finance_controller
                                .update_category(fm_core::Category { id, name })
                                .await?
                                .id,
                        ))
                    } else {
                        Ok(Message::CategoryCreated(
                            finance_controller.create_category(name).await?.id,
                        ))
                    }
                }))
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        components::spaced_column![
            components::labeled_entry("Name", "", &self.name, Some(Message::NameInput)),
            components::submit_cancel_row(
                if self.is_submittable() {
                    Some(Message::Submit)
                } else {
                    None
                },
                Some(Message::Cancel)
            ),
        ]
        .into()
    }

    fn is_submittable(&self) -> bool {
        self.name.is_valid()
    }
}
