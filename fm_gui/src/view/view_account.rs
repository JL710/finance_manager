use super::super::{AppMessage, View};
use fm_core;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub struct ViewAccount {
    account: fm_core::account::Account,
    transactions: Vec<fm_core::Transaction>,
    current_value: fm_core::Currency,
}

impl ViewAccount {
    pub fn new(account: fm_core::account::Account, account_sum: fm_core::Currency) -> Self {
        Self {
            current_value: account_sum, // finance_manager.get_account_sum(&account, chrono::Utc::now()),
            account,
            transactions: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        _message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        match &self.account {
            fm_core::account::Account::AssetAccount(acc) => {
                asset_account_view(acc, &self.transactions, &self.current_value)
            }
            _ => iced::widget::text("comming soon").into(),
        }
    }
}

fn asset_account_view<'a>(
    account: &fm_core::account::AssetAccount,
    transactions: &[fm_core::Transaction],
    current_value: &fm_core::Currency,
) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
    let mut transactions_table = super::super::table::Table::<'_, Message>::new(2);

    for transaction in transactions {
        // TODO: push transaction
    }

    iced::widget::column![
        iced::widget::text(format!("Account: {}", account.name())),
        iced::widget::text(format!("Notes: {}", account.note().unwrap_or(""))),
        iced::widget::text(format!("IBAN: {}", account.iban().unwrap_or(""))),
        iced::widget::text(format!("BIC/Swift: {}", account.bic().unwrap_or(""))),
        iced::widget::text(format!("Current Amount: {}", current_value)),
        iced::widget::horizontal_rule(10),
        transactions_table.convert_to_view()
    ]
    .into()
}
