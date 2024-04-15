use super::AppMessage;
use crate::finance;

#[derive(Debug, Clone)]
pub struct ViewAccount {
    account: finance::account::Account,
    transactions: Vec<finance::Transaction>,
    current_value: finance::Currency,
}

impl ViewAccount {
    pub fn new(
        finance_manager: &finance::FinanceManager,
        account: finance::account::Account,
    ) -> Self {
        Self {
            current_value: finance_manager.get_account_sum(&account, chrono::Utc::now()),
            account,
            transactions: Vec::new(),
        }
    }

    pub fn update(&mut self, finance_manager: &finance::FinanceManager) {}

    pub fn view(&self) -> iced::Element<'_, AppMessage, iced::Theme, iced::Renderer> {
        match &self.account {
            finance::account::Account::AssetAccount(acc) => {
                asset_account_view(acc, &self.transactions, &self.current_value)
            }
            _ => iced::widget::text("comming soon").into(),
        }
    }
}

fn asset_account_view<'a>(
    account: &finance::account::AssetAccount,
    transactions: &[finance::Transaction],
    current_value: &finance::Currency,
) -> iced::Element<'a, AppMessage, iced::Theme, iced::Renderer> {
    iced::widget::column![
        iced::widget::text(format!("Account: {}", account.name())),
        iced::widget::text(format!("Notes: {}", account.note().unwrap_or(""))),
        iced::widget::text(format!("IBAN: {}", account.iban().unwrap_or(""))),
        iced::widget::text(format!("BIC/Swift: {}", account.bic().unwrap_or(""))),
        iced::widget::text(format!("Current Amount: {}", current_value)),
        iced::widget::horizontal_rule(10)
    ]
    .into()
}
