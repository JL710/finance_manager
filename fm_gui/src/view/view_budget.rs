use super::super::{AppMessage, View};
use anyhow::Result;
use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(BudgetView::fetch(id, finance_manager), |result| {
        AppMessage::SwitchView(View::ViewBudgetView(result.unwrap()))
    })
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub struct BudgetView {
    budget: fm_core::Budget,
    current_value: fm_core::Currency,
    transactions: Vec<fm_core::Transaction>,
}

impl BudgetView {
    pub fn new(
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transactions: Vec<fm_core::Transaction>,
    ) -> Self {
        Self {
            budget,
            current_value,
            transactions,
        }
    }

    pub async fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let budget = locked_manager.get_budget(id).await?.unwrap();
        let transactions = locked_manager
            .get_current_budget_transactions(&budget)
            .await?;
        let mut current_value = fm_core::Currency::Eur(0.0);
        for transaction in &transactions {
            current_value += transaction.amount();
        }

        Ok(Self::new(budget, current_value, transactions))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
        widget::text("todo").into()
    }
}
