use iced::widget;

pub enum Action {
    None,
    ViewBudget(fm_core::Id),
    CreateBudget,
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateBudget,
    ViewBudget(fm_core::Id),
    Initialize(Vec<(fm_core::Budget, fm_core::Currency)>),
    Reload(Vec<(fm_core::Budget, fm_core::Currency)>),
    BudgetTable(components::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct View {
    budgets: Vec<(fm_core::Budget, fm_core::Currency)>,
    budget_table: components::table_view::State<(fm_core::Budget, fm_core::Currency), ()>,
}

impl View {
    pub fn new(budgets: Vec<(fm_core::Budget, fm_core::Currency)>) -> Self {
        Self {
            budgets: budgets.clone(),
            budget_table: components::table_view::State::new(budgets, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name.cmp(&b.0.name),
                    1 => a.1.cmp(&b.1),
                    2 => a.0.total_value.cmp(&b.0.total_value),
                    _ => panic!(),
                })
                .sortable_columns([0, 1, 2]),
        }
    }

    pub fn reload(
        &mut self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<Message> {
        error::failing_task(async move {
            let budgets = finance_controller.get_budgets().await?;
            let mut tuples = Vec::new();

            for budget in budgets {
                let current_value = finance_controller
                    .get_budget_value(&budget, 0, utc_offset)
                    .await?;
                tuples.push((budget, current_value));
            }

            Ok(Message::Reload(tuples))
        })
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> (Self, iced::Task<Message>) {
        (
            View::new(Vec::new()),
            error::failing_task(async move {
                let budgets = finance_controller.get_budgets().await?;
                let mut tuples = Vec::new();

                for budget in budgets {
                    let current_value = finance_controller
                        .get_budget_value(&budget, 0, utc_offset)
                        .await?;
                    tuples.push((budget, current_value));
                }

                Ok(Message::Initialize(tuples))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Reload(budgets) => {
                self.budgets = budgets.clone();
                self.budget_table.edit_items(|items| *items = budgets);
                Action::None
            }
            Message::CreateBudget => Action::CreateBudget,
            Message::ViewBudget(id) => Action::ViewBudget(id),
            Message::Initialize(budgets) => {
                self.budgets = budgets.clone();
                self.budget_table.set_items(budgets);
                Action::None
            }
            Message::BudgetTable(inner) => match self.budget_table.perform(inner) {
                components::table_view::Action::OuterMessage(m) => {
                    self.update(m, _finance_controller)
                }
                components::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::BudgetTable))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        components::overlap_bottom_right(
            components::table_view::table_view(&self.budget_table)
                .headers([
                    "Name".to_string(),
                    "Current".to_string(),
                    "Total".to_string(),
                ])
                .view(|budget, _| {
                    [
                        components::link(budget.0.name.as_str())
                            .on_press(Message::ViewBudget(budget.0.id))
                            .into(),
                        widget::text!("{}", &budget.1).into(),
                        widget::text!("{}", budget.0.total_value).into(),
                    ]
                })
                .map(Message::BudgetTable),
            components::button::large_round_plus_button(Some(Message::CreateBudget)),
        )
        .height(iced::Fill)
        .into()
    }
}
