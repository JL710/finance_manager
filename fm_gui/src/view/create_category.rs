use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    Submit,
    NameInput(String),
}

#[derive(Debug, Clone, Default)]
pub struct CreateCategory {
    id: Option<fm_core::Id>,
    name: String,
}

impl CreateCategory {
    pub fn new(id: Option<fm_core::Id>, name: String) -> Self {
        Self { id, name }
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::NameInput(content) => self.name = content,
            Message::Submit => {
                let id = self.id;
                let name = self.name.clone();
                let manager = finance_manager.clone();
                return (
                    None,
                    iced::Task::future(async move {
                        let mut locked_manager = finance_manager.lock().await;
                        if let Some(id) = id {
                            locked_manager.update_category(id, name).await.unwrap();
                        } else {
                            locked_manager.create_category(name).await.unwrap();
                        }
                        ()
                    })
                    .then(move |_| {
                        let (view, task) =
                            super::category_overview::CategoryOverview::fetch(manager.clone());
                        iced::Task::done(AppMessage::SwitchView(View::CategoryOverview(view)))
                            .chain(task.map(AppMessage::CategoryOverviewMessage))
                    }),
                );
            }
        }
        (None, iced::Task::none())
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
