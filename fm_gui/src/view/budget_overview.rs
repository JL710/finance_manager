use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    ViewBudget(fm_core::Id),
    CreateBudget,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateBudget,
    ViewBudget(fm_core::Id),
    Initialize(Vec<(fm_core::Budget, fm_core::Currency)>),
    BudgetTable(utils::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct BudgetOverview {
    budgets: Vec<(fm_core::Budget, fm_core::Currency)>,
    budget_table: utils::table_view::State<(fm_core::Budget, fm_core::Currency), ()>,
}

impl BudgetOverview {
    pub fn new(budgets: Vec<(fm_core::Budget, fm_core::Currency)>) -> Self {
        Self {
            budgets: budgets.clone(),
            budget_table: utils::table_view::State::new(budgets, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name().cmp(b.0.name()),
                    1 => a.1.cmp(&b.1),
                    2 => a.0.total_value().cmp(&b.0.total_value()),
                    _ => panic!(),
                })
                .sortable_columns([0, 1, 2]),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            BudgetOverview::new(Vec::new()),
            iced::Task::future(async move {
                let budgets = finance_manager.lock().await.get_budgets().await.unwrap();
                let mut tuples = Vec::new();

                for budget in budgets {
                    let current_value = finance_manager
                        .lock()
                        .await
                        .get_budget_value(&budget, 0)
                        .unwrap()
                        .await
                        .unwrap();
                    tuples.push((budget, current_value));
                }

                Message::Initialize(tuples)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::CreateBudget => Action::CreateBudget,
            Message::ViewBudget(id) => Action::ViewBudget(id),
            Message::Initialize(budgets) => {
                self.budgets = budgets.clone();
                self.budget_table.set_items(budgets);
                Action::None
            }
            Message::BudgetTable(inner) => {
                if let utils::table_view::Action::OuterMessage(m) = self.budget_table.perform(inner)
                {
                    return self.update(m, _finance_manager);
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::column![
            utils::heading("Budget Overview", utils::HeadingLevel::H1),
            widget::button::Button::new("Create Budget").on_press(Message::CreateBudget),
            widget::horizontal_rule(10),
            utils::table_view::table_view(&self.budget_table)
                .headers([
                    "Name".to_string(),
                    "Current".to_string(),
                    "Total".to_string(),
                ])
                .view(|budget, _| {
                    [
                        utils::link(widget::text(budget.0.name().to_string()))
                            .on_press(Message::ViewBudget(*budget.0.id()))
                            .into(),
                        widget::text!("{}", &budget.1).into(),
                        widget::text!("{}", budget.0.total_value()).into(),
                    ]
                })
                .map(Message::BudgetTable),
        ]
        .height(iced::Fill)
        .spacing(10)
        .into()
    }
}
