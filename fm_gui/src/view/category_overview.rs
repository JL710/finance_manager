use super::super::utils;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    ViewCategory(fm_core::Id),
    NewCategory,
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewCategory(fm_core::Id),
    NewCategory,
    Initialize(Vec<fm_core::Category>),
}

#[derive(Debug, Clone)]
pub struct CategoryOverview {
    categories: Vec<fm_core::Category>,
}

impl CategoryOverview {
    pub fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                categories: Vec::new(),
            },
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let categories = locked_manager.get_categories().await.unwrap();
                Message::Initialize(categories)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::NewCategory => Action::NewCategory,
            Message::ViewCategory(category_id) => Action::ViewCategory(category_id),
            Message::Initialize(categories) => {
                self.categories = categories;
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            utils::heading("Category Overview", utils::HeadingLevel::H1),
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
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
