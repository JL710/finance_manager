use super::super::{utils, AppMessage, View};
use fm_core::{self, FinanceManager};
use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    let (view, task) = BudgetOverview::fetch(finance_manager.clone());
    iced::Task::done(AppMessage::SwitchView(View::BudgetOverview(view)))
        .chain(task.map(AppMessage::BudgetOverViewMessage))
}

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
}

#[derive(Debug, Clone)]
pub struct BudgetOverview {
    budgets: Vec<(fm_core::Budget, fm_core::Currency)>,
}

impl BudgetOverview {
    pub fn new(budgets: Vec<(fm_core::Budget, fm_core::Currency)>) -> Self {
        Self { budgets }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                budgets: Vec::new(),
            },
            iced::Task::future(async move {
                let budgets = finance_manager.lock().await.get_budgets().await.unwrap();
                let mut tuples = Vec::new();

                for budget in budgets {
                    let current_value = finance_manager
                        .lock()
                        .await
                        .get_budget_value(&budget, 0)
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
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::CreateBudget => Action::CreateBudget,
            Message::ViewBudget(id) => Action::ViewBudget(id),
            Message::Initialize(budgets) => {
                self.budgets = budgets;
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let budget_table = utils::TableView::new(self.budgets.clone(), |budget| {
            [
                utils::link(widget::text(budget.0.name().to_string()))
                    .on_press(Message::ViewBudget(*budget.0.id()))
                    .into(),
                widget::text!("{}", &budget.1).into(),
                widget::text!("{}", budget.0.total_value()).into(),
            ]
        })
        .headers([
            "Name".to_string(),
            "Current".to_string(),
            "Total".to_string(),
        ])
        .sort_by(|a, b, column| match column {
            0 => a.0.name().cmp(b.0.name()),
            1 => a.1.cmp(&b.1),
            2 => a.0.total_value().cmp(&b.0.total_value()),
            _ => panic!(),
        })
        .columns_sortable([true, true, true]);
        widget::column![
            utils::heading("Budget Overview", utils::HeadingLevel::H1),
            widget::button::Button::new("Create Budget").on_press(Message::CreateBudget),
            widget::horizontal_rule(10),
            budget_table.into_element(),
        ]
        .spacing(10)
        .into()
    }
}
