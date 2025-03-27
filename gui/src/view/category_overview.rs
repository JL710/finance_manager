pub enum Action {
    None,
    ViewCategory(fm_core::Id),
    NewCategory,
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewCategory(fm_core::Id),
    NewCategory,
    Initialize(Vec<fm_core::Category>),
    CategoryTable(components::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct View {
    category_table: components::table_view::State<fm_core::Category, ()>,
}

impl View {
    pub fn new(categories: Vec<fm_core::Category>) -> Self {
        Self {
            category_table: components::table_view::State::new(categories, ())
                .sort_by(|a, b, _| a.name.cmp(&b.name))
                .sortable_columns([0]),
        }
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new()),
            error::failing_task(async move {
                let categories = finance_controller.get_categories().await?;
                Ok(Message::Initialize(categories))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::NewCategory => Action::NewCategory,
            Message::ViewCategory(category_id) => Action::ViewCategory(category_id),
            Message::Initialize(categories) => {
                self.category_table.set_items(categories);
                Action::None
            }
            Message::CategoryTable(inner) => match self.category_table.perform(inner) {
                components::table_view::Action::OuterMessage(m) => {
                    self.update(m, _finance_controller)
                }
                components::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::CategoryTable))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Category Overview",
            components::spaced_column![
                components::button::new("New Category", Some(Message::NewCategory)),
                iced::widget::horizontal_rule(10),
                components::table_view::table_view(&self.category_table)
                    .headers(["Name".to_string()])
                    .view(|category, _| [components::link(category.name.as_str())
                        .on_press(Message::ViewCategory(category.id))
                        .into()])
                    .map(Message::CategoryTable),
            ]
            .height(iced::Fill)
            .width(iced::Fill),
        )
    }
}
