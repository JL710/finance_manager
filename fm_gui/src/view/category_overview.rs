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
    CategoryTable(utils::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct CategoryOverview {
    category_table: utils::table_view::State<fm_core::Category, ()>,
}

impl CategoryOverview {
    pub fn new(categories: Vec<fm_core::Category>) -> Self {
        Self {
            category_table: utils::table_view::State::new(categories, ())
                .sort_by(|a, b, _| a.name().cmp(b.name()))
                .sortable_columns([0]),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new()),
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
        _finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::NewCategory => Action::NewCategory,
            Message::ViewCategory(category_id) => Action::ViewCategory(category_id),
            Message::Initialize(categories) => {
                self.category_table.set_items(categories);
                Action::None
            }
            Message::CategoryTable(inner) => {
                if let utils::table_view::Action::OuterMessage(m) =
                    self.category_table.perform(inner)
                {
                    return self.update(m, _finance_manager);
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            utils::heading("Category Overview", utils::HeadingLevel::H1),
            iced::widget::button("New Category").on_press(Message::NewCategory),
            iced::widget::horizontal_rule(10),
            utils::table_view::table_view(&self.category_table)
                .headers(["Name".to_string()])
                .view(
                    |category, _| [utils::link(iced::widget::text(category.name().to_string()))
                        .on_press(Message::ViewCategory(*category.id()))
                        .into()]
                )
                .map(Message::CategoryTable),
        ]
        .spacing(10)
        .height(iced::Fill)
        .width(iced::Fill)
        .into()
    }
}
