use super::super::{AppMessage, View};

use anyhow::Result;
use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    category_id: fm_core::Id,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            View::ViewCategory(
                ViewCategory::fetch(finance_manager, category_id)
                    .await
                    .unwrap(),
            )
        },
        AppMessage::SwitchView,
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Delete,
    Edit,
    ChangedTimespan(fm_core::Timespan),
}

#[derive(Debug, Clone)]
pub struct ViewCategory {
    category: fm_core::Category,
}

impl ViewCategory {
    pub fn new(category: fm_core::Category) -> Self {
        Self { category }
    }

    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        category_id: fm_core::Id,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        Ok(Self {
            category: locked_manager
                .get_category(category_id)
                .await
                .unwrap()
                .unwrap(),
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Delete => {
                let category_id = *self.category.id();
                (
                    None,
                    iced::Command::perform(
                        async move {
                            finance_manager
                                .lock()
                                .await
                                .delete_category(category_id)
                                .await
                                .unwrap();
                            AppMessage::SwitchView(View::CategoryOverview(
                                super::category_overview::CategoryOverview::fetch(finance_manager)
                                    .await
                                    .unwrap(),
                            ))
                        },
                        |msg| msg,
                    ),
                )
            }
            Message::Edit => (
                Some(View::CreateCategory(
                    super::create_category::CreateCategory::from_existing(
                        Some(*self.category.id()),
                        self.category.name().to_string(),
                    ),
                )),
                iced::Command::none(),
            ),
            Message::ChangedTimespan(timespan) => {
                todo!()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::column![
            widget::row![
                widget::text(self.category.name().to_string()),
                widget::button("Edit").on_press(Message::Edit),
                widget::button("Delete").on_press(Message::Delete),
            ]
            .spacing(10),
            super::super::timespan_input::TimespanInput::new(Message::ChangedTimespan)
                .into_element()
        ]
        .into()
    }
}
