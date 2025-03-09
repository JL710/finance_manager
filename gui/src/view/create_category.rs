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
}

#[derive(Debug, Clone, Default)]
pub struct View {
    id: Option<fm_core::Id>,
    name: String,
    submitted: bool,
}

impl View {
    pub fn new(id: Option<fm_core::Id>, name: String) -> Self {
        Self {
            id,
            name,
            submitted: false,
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(None, String::new()),
            utils::failing_task(async move {
                let category = finance_controller
                    .get_category(id)
                    .await?
                    .context(format!("Could not find category {}", id))?;
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
                self.name = category.name;
                Action::None
            }
            Message::NameInput(content) => {
                self.name = content;
                Action::None
            }
            Message::Submit => {
                self.submitted = true;
                let id = self.id;
                let name = self.name.clone();
                Action::Task(utils::failing_task(async move {
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

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Create Category",
            utils::spaced_column![
                utils::labeled_entry("Name", &self.name, Message::NameInput, true),
                utils::submit_cancel_row(
                    if self.is_submittable() {
                        Some(Message::Submit)
                    } else {
                        None
                    },
                    Some(Message::Cancel)
                ),
            ],
        )
    }

    fn is_submittable(&self) -> bool {
        !self.name.is_empty()
    }
}
