use super::super::utils;

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    None,
    CategoryCreated(fm_core::Id),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    NameInput(String),
    CategoryCreated(fm_core::Id),
    Initialize(fm_core::Category),
}

#[derive(Debug, Clone, Default)]
pub struct CreateCategory {
    id: Option<fm_core::Id>,
    name: String,
    submitted: bool,
}

impl CreateCategory {
    pub fn new(id: Option<fm_core::Id>, name: String) -> Self {
        Self {
            id,
            name,
            submitted: false,
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(None, String::new()),
            iced::Task::future(async move {
                let category = finance_manager
                    .lock()
                    .await
                    .get_category(id)
                    .await
                    .unwrap()
                    .unwrap();
                Message::Initialize(category)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
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
                Action::Task(iced::Task::future(async move {
                    let mut locked_manager = finance_manager.lock().await;
                    if let Some(id) = id {
                        Message::CategoryCreated(
                            *locked_manager.update_category(id, name).await.unwrap().id(),
                        )
                    } else {
                        Message::CategoryCreated(
                            *locked_manager.create_category(name).await.unwrap().id(),
                        )
                    }
                }))
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
            utils::heading("Create Category", utils::HeadingLevel::H1),
            widget::row![
                widget::text("Name:"),
                widget::text_input("Name", &self.name).on_input(Message::NameInput),
            ]
            .spacing(10),
            widget::button("Submit").on_press_maybe(if self.is_submittable() {
                Some(Message::Submit)
            } else {
                None
            })
        ]
        .spacing(10)
        .into()
    }

    fn is_submittable(&self) -> bool {
        !self.name.is_empty()
    }
}
