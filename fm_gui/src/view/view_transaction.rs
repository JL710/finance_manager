use anyhow::Result;

use std::sync::Arc;
use tokio::sync::Mutex;

use iced::widget;

use super::super::{AppMessage, View};

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move { TransactionView::fetch(id, finance_manager).await.unwrap() },
        |x| AppMessage::SwitchView(View::TransactionView(x)),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit,
}

#[derive(Debug, Clone)]
pub struct TransactionView {
    transaction: fm_core::Transaction,
    source: fm_core::account::Account,
    destination: fm_core::account::Account,
    budget: Option<fm_core::Budget>,
}

impl TransactionView {
    pub fn new(
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
    ) -> Self {
        Self {
            transaction,
            source,
            destination,
            budget,
        }
    }

    pub async fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let transaction = locked_manager.get_transaction(id).await?.unwrap();
        let source = locked_manager
            .get_account(*transaction.source())
            .await?
            .unwrap();
        let destination = locked_manager
            .get_account(*transaction.destination())
            .await?
            .unwrap();
        let budget = match transaction.budget() {
            Some(budget_id) => locked_manager.get_budget(*budget_id).await?,
            None => None,
        };
        Ok(Self::new(transaction, source, destination, budget))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Edit => (
                Some(View::Empty),
                super::create_transaction::edit_switch_view_command(
                    *self.transaction.id(),
                    finance_manager,
                ),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
        let mut column = widget::column![
            widget::text(format!("Value: {}", self.transaction.amount())),
            widget::text(format!("Name: {}", self.transaction.title())),
            widget::text(format!("Source: {}", self.source)),
            widget::text(format!("Destination: {}", self.destination)),
            widget::text(format!(
                "Date: {}",
                self.transaction.date().format("%d.%m.%Y")
            ))
        ]
        .spacing(10);

        if let Some(budget) = &self.budget {
            column = column.push(widget::text(format!("Budget: {}", budget.name())));
        }

        if let Some(content) = self.transaction.description() {
            column = column.push(widget::text(format!("Description: {}", content)));
        }

        widget::row![
            column,
            widget::Space::with_width(iced::Length::Fill),
            widget::button("Edit").on_press(Message::Edit)
        ]
        .into()
    }
}
