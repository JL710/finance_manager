use super::super::{utils, AppMessage, View};

use anyhow::Result;
use async_std::sync::Mutex;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
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
    pub fn new(categories: Vec<fm_core::Category>) -> Self {
        Self { categories }
    }

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
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::NewCategory => (
                Some(View::CreateCategory(
                    super::create_category::CreateCategory::new(),
                )),
                iced::Command::none(),
            ),
            Message::ViewCategory(category_id) => (
                None,
                super::view_category::switch_view_command(finance_manager, category_id),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let table = super::super::table_view::TableView::new(self.categories.clone(), |category| {
            [
                iced::widget::button(iced::widget::text(category.name().to_string()))
                    .on_press(Message::ViewCategory(*category.id()))
                    .style(utils::button_link_style)
                    .padding(0)
                    .into(),
            ]
        })
        .headers(["Name".to_string()]);

        iced::widget::column![
            iced::widget::button("New Category").on_press(Message::NewCategory),
            table.into_element()
        ]
        .width(iced::Length::Fill)
        .spacing(10)
        .into()
    }
}
