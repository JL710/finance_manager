use anyhow::Context;
use async_std::sync::Mutex;
use std::sync::Arc;

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
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(None, String::new()),
            utils::failing_task(async move {
                let category = finance_manager
                    .lock()
                    .await
                    .get_category(id)
                    .await
                    .context(format!("Error while fetching category {}", id))?
                    .context(format!("Could not find category {}", id))?;
                Ok(Message::Initialize(category))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
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
                self.id = Some(*category.id());
                self.name = category.name().to_string();
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
                    let mut locked_manager = finance_manager.lock().await;
                    if let Some(id) = id {
                        Ok(Message::CategoryCreated(
                            *locked_manager
                                .update_category(id, name)
                                .await
                                .context(format!("Error while updating category {}", id))?
                                .id(),
                        ))
                    } else {
                        Ok(Message::CategoryCreated(
                            *locked_manager
                                .create_category(name)
                                .await
                                .context("Error while creating category")?
                                .id(),
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
