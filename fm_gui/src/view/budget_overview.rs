use super::super::{utils, AppMessage, View};
use fm_core::{self, FinanceManager};
use iced::widget;

use anyhow::Result;
use async_std::sync::Mutex;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
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
            let current_value = locked_manager
                .get_current_budget_value(&budget)
                .await
                .unwrap();
            tuples.push((budget, current_value));
        }

        Ok(BudgetOverview::new(tuples))
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::CreateBudget => (
                Some(View::CreateBudgetView(
                    super::create_budget::CreateBudgetView::default(),
                )),
                iced::Command::none(),
            ),
            Message::ViewBudget(id) => (
                Some(View::Empty),
                super::view_budget::switch_view_command(id, _finance_manager),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::column![
            widget::button::Button::new("Create Budget").on_press(Message::CreateBudget),
            widget::horizontal_rule(10),
            generate_budget_list(&self.budgets),
        ]
        .into()
    }
}

fn generate_budget_list(
    budgets: &Vec<(fm_core::Budget, fm_core::Currency)>,
) -> iced::Element<'_, Message> {
    let mut budget_table = super::super::table::Table::new(3).set_headers(vec![
        "Name".to_string(),
        "Current".to_string(),
        "Total".to_string(),
    ]);

    for budget in budgets {
        budget_table.push_row(vec![
            widget::button(budget.0.name())
                .on_press(Message::ViewBudget(*budget.0.id()))
                .padding(0)
                .style(utils::button_link_style)
                .into(),
            widget::text(format!("{}", &budget.1)).into(),
            widget::text(format!("{}", budget.0.total_value())).into(),
        ]);
    }

    budget_table.convert_to_view()
}
