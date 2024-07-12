use super::super::{utils, AppMessage, View};

use anyhow::Result;
use async_std::sync::Mutex;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
        async move { View::CategoryOverview(CategoryOverview::fetch(finance_manager).await.unwrap()) },
        AppMessage::SwitchView,
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewCategory(fm_core::Id),
    NewCategory,
}

#[derive(Debug, Clone)]
pub struct CategoryOverview {
    categories: Vec<fm_core::Category>,
}

impl CategoryOverview {
    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        Ok(Self {
            categories: locked_manager.get_categories().await.unwrap(),
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::NewCategory => (
                Some(View::CreateCategory(
                    super::create_category::CreateCategory::new(),
                )),
                iced::Task::none(),
            ),
            Message::ViewCategory(category_id) => (
                None,
                super::category::switch_view_command(finance_manager, category_id),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            iced::widget::button("New Category").on_press(Message::NewCategory),
            iced::widget::horizontal_rule(10),
            utils::TableView::new(self.categories.clone(), |category| {
                [utils::link(iced::widget::text(category.name().to_string()))
                    .on_press(Message::ViewCategory(*category.id()))
                    .into()]
            })
            .headers(["Name".to_string()])
            .sort_by(|a, b, _| a.name().cmp(b.name()))
            .columns_sortable([true])
            .into_element()
        ]
        .width(iced::Length::Fill)
        .into()
    }
}
