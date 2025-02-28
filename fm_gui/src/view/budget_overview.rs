use iced::widget;

use anyhow::Context;
use async_std::sync::Mutex;
use std::sync::Arc;

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
    BudgetTable(utils::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct View {
    budgets: Vec<(fm_core::Budget, fm_core::Currency)>,
    budget_table: utils::table_view::State<(fm_core::Budget, fm_core::Currency), ()>,
}

impl View {
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
            View::new(Vec::new()),
            utils::failing_task(async move {
                let budgets = finance_manager.lock().await.get_budgets().await?;
                let mut tuples = Vec::new();

                for budget in budgets {
                    let current_value = finance_manager
                        .lock()
                        .await
                        .get_budget_value(
                            &budget,
                            0,
                            fm_core::get_local_timezone()
                                .context("Error while trying to get local timezone")?,
                        )
                        .context(format!(
                            "Error while trying to fetch value of budget {} {}",
                            budget.id(),
                            budget.name()
                        ))?
                        .await
                        .context(format!(
                            "Error while fetching budget value for budget {} {}",
                            budget.id(),
                            budget.name()
                        ))?;
                    tuples.push((budget, current_value));
                }

                Ok(Message::Initialize(tuples))
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
            Message::BudgetTable(inner) => match self.budget_table.perform(inner) {
                utils::table_view::Action::OuterMessage(m) => self.update(m, _finance_manager),
                utils::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::BudgetTable))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        super::view(
            "Budget Overview",
            utils::spaced_column![
                utils::button::new("Create Budget", Some(Message::CreateBudget)),
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
            .height(iced::Fill),
        )
    }
}
