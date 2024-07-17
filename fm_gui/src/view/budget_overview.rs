use super::super::{utils, AppMessage, View};
use fm_core::{self, FinanceManager};
use iced::widget;

use anyhow::Result;
use async_std::sync::Mutex;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
        async move { BudgetOverview::fetch(finance_manager).await.unwrap() },
        |x| AppMessage::SwitchView(View::BudgetOverview(x)),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateBudget,
    ViewBudget(fm_core::Id),
}

#[derive(Debug, Clone)]
pub struct BudgetOverview {
    budgets: Vec<(fm_core::Budget, fm_core::Currency)>,
}

impl BudgetOverview {
    pub fn new(budgets: Vec<(fm_core::Budget, fm_core::Currency)>) -> Self {
        Self { budgets }
    }

    pub async fn fetch(finance_manager: Arc<Mutex<impl FinanceManager + 'static>>) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let budgets = locked_manager.get_budgets().await.unwrap();
        let mut tuples = Vec::new();

        for budget in budgets {
            let current_value = locked_manager.get_budget_value(&budget, 0).await.unwrap();
            tuples.push((budget, current_value));
        }

        Ok(BudgetOverview::new(tuples))
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::CreateBudget => (
                Some(View::CreateBudgetView(
                    super::create_budget::CreateBudgetView::default(),
                )),
                iced::Task::none(),
            ),
            Message::ViewBudget(id) => (
                Some(View::Empty),
                super::budget::switch_view_command(id, _finance_manager),
            ),
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
