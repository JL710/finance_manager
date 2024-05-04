use super::super::{AppMessage, View};

use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    NameInput(String),
}

#[derive(Debug, Clone)]
pub struct CreateCategory {
    id: Option<fm_core::Id>,
    name: String,
}

impl CreateCategory {
    pub fn new() -> Self {
        Self {
            id: None,
            name: String::new(),
        }
    }

    pub fn from_existing(id: Option<fm_core::Id>, name: String) -> Self {
        Self { id, name }
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::NameInput(content) => self.name = content,
            Message::Submit => {
                let id = self.id;
                let name = self.name.clone();
                return (
                    None,
                    iced::Command::perform(
                        async move {
                            let mut locked_manager = finance_manager.lock().await;
                            if let Some(id) = id {
                                locked_manager.update_category(id, name).await.unwrap();
                            } else {
                                locked_manager.create_category(name).await.unwrap();
                            }
                            drop(locked_manager);

                            super::category_overview::CategoryOverview::fetch(finance_manager)
                                .await
                                .unwrap()
                        },
                        |x| AppMessage::SwitchView(View::CategoryOverview(x)),
                    ),
                );
            }
        }
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
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
