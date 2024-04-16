use super::super::AppMessage;
use crate::finance;

use super::View;

#[derive(Debug, Clone)]
pub struct ViewAccount {
    account: finance::account::Account,
    transactions: Vec<finance::Transaction>,
    current_value: finance::Currency,
}

impl View for ViewAccount {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        _message: Self::ParentMessage,
        _finance_manager: &mut finance::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>> {
        None
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        self.view()
    }
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
    let mut transactions_table = super::super::table::Table::<'_, AppMessage>::new(2);

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
