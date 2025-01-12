use async_std::sync::Mutex;
use iced::widget;
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
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
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
            utils::labeled_entry("Name", &self.name, Message::NameInput, true),
            widget::row![
                widget::button("Cancel")
                    .on_press(Message::Cancel)
                    .style(widget::button::danger),
                widget::horizontal_space(),
                widget::button("Submit")
                    .on_press_maybe(if self.is_submittable() {
                        Some(Message::Submit)
                    } else {
                        None
                    })
                    .style(widget::button::success)
            ],
        ]
        .spacing(10)
        .into()
    }

    fn is_submittable(&self) -> bool {
        !self.name.is_empty()
    }
}
