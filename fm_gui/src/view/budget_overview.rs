use super::super::{utils, AppMessage, View};
use fm_core::{self, FinanceManager};
use iced::widget;

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move { finance_manager.lock().await.get_budgets().await },
        |budgets| AppMessage::SwitchView(View::BudgetOverview(BudgetOverview::new(budgets))),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateBudget,
}

#[derive(Debug, Clone)]
pub struct BudgetOverview {
    budgets: Vec<fm_core::Budget>,
}

impl BudgetOverview {
    pub fn new(budgets: Vec<fm_core::Budget>) -> Self {
        Self { budgets }
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::CreateBudget => (
                Some(View::CreateBudgetView(
                    super::create_budget::CreateBudgetView::new(),
                )),
                iced::Command::none(),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        widget::column![
            widget::button::Button::new("Create Budget").on_press(Message::CreateBudget),
            widget::horizontal_rule(10),
            generate_budget_list(&self.budgets),
        ]
        .into()
    }
}

fn generate_budget_entry(
    budget: &fm_core::Budget,
) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
    widget::container(widget::text(budget.name()))
        .style(utils::entry_row_container_style_weak)
        .into()
}

fn generate_budget_list(
    budgets: &Vec<fm_core::Budget>,
) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
    let mut column = widget::Column::new();

    for budget in budgets {
        column = column.push(generate_budget_entry(budget));
    }

    widget::scrollable(column).into()
}
