use super::super::{utils, AppMessage, View};
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
pub enum Message {
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub struct BudgetView {
    budget: fm_core::Budget,
    current_value: fm_core::Currency,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
}

impl BudgetView {
    pub fn new(
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
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

        let mut transactop_tuples = Vec::new();
        for transaction in transactions {
            let source = locked_manager
                .get_account(*transaction.source())
                .await?
                .unwrap();
            let destination = locked_manager
                .get_account(*transaction.destination())
                .await?
                .unwrap();
            transactop_tuples.push((transaction, source, destination));
        }

        Ok(Self::new(budget, current_value, transactop_tuples))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::ViewAccount(id) => (
                Some(View::Empty),
                super::view_account::switch_view_command(id, finance_manager),
            ),
            Message::ViewTransaction(id) => (
                Some(View::Empty),
                super::view_transaction::switch_view_command(id, finance_manager),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        let mut column = widget::column![
            widget::text(format!("Name: {}", self.budget.name())),
            widget::text(format!("Current Value: {}", self.current_value)),
            widget::text(format!("Recouring: {}", self.budget.timespan())),
            widget::progress_bar(
                0.0..=self.budget.total_value().get_num() as f32,
                self.current_value.get_num() as f32
            )
        ]
        .spacing(10);

        if let Some(content) = self.budget.description() {
            column = column.push(widget::text(format!("Description: {}", content)));
        }

        column = column.push(utils::transaction_table(
            &self.transactions,
            |_| None,
            Message::ViewTransaction,
            Message::ViewAccount,
        ));

        column.into()
    }
}
