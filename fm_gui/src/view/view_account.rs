use super::super::{utils, AppMessage, View};

use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    account_id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move { View::ViewAccount(ViewAccount::fetch(finance_manager, account_id).await) },
        AppMessage::SwitchView,
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit,
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    ChangeTransactionTimespan(fm_core::Timespan),
    SetTransactions(
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
}

#[derive(Debug, Clone)]
pub struct ViewAccount {
    account: fm_core::account::Account,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    current_value: fm_core::Currency,
}

impl ViewAccount {
    pub fn new(
        account: fm_core::account::Account,
        account_sum: fm_core::Currency,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ) -> Self {
        Self {
            current_value: account_sum, // finance_manager.get_account_sum(&account, chrono::Utc::now()),
            account,
            transactions,
        }
    }

    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        account_id: fm_core::Id,
    ) -> Self {
        let locked_manager = finance_manager.lock().await;
        let account = locked_manager
            .get_account(account_id)
            .await
            .unwrap()
            .unwrap();
        let account_sum = locked_manager
            .get_account_sum(&account, chrono::Utc::now())
            .await
            .unwrap();
        let mut transactions = locked_manager
            .get_transactions_of_account(*account.id(), (None, Some(chrono::Utc::now())))
            .await
            .unwrap();
        transactions.sort_by(|a, b| b.date().cmp(a.date())); // FIXME: this should not be related to the view -> the table should do it
        let accounts = locked_manager.get_accounts_hash_map().await.unwrap();
        let mut transaction_tuples = Vec::with_capacity(transactions.len());
        for transaction in transactions {
            let source = accounts.get(transaction.source()).unwrap().clone();
            let destination = accounts.get(transaction.destination()).unwrap().clone();
            transaction_tuples.push((transaction, source, destination));
        }
        Self::new(account, account_sum, transaction_tuples)
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Edit => match &self.account {
                fm_core::account::Account::AssetAccount(acc) => {
                    return (
                            Some(View::CreateAssetAccountDialog(
                                super::create_asset_account::CreateAssetAccountDialog::from_existing_account(acc)
                            )), iced::Command::none());
                }
                fm_core::account::Account::BookCheckingAccount(acc) => todo!(),
            },
            Message::ViewTransaction(id) => {
                return (
                    Some(View::Empty),
                    super::view_transaction::switch_view_command(id, finance_manager),
                )
            }
            Message::ViewAccount(id) => {
                (Some(View::Empty), switch_view_command(id, finance_manager))
            }
            Message::SetTransactions(transactions) => {
                self.transactions = transactions;
                (None, iced::Command::none())
            }
            Message::ChangeTransactionTimespan(timespan) => {
                let account_id = *self.account.id();
                (
                    None,
                    iced::Command::perform(
                        async move {
                            let locked_manager = finance_manager.lock().await;
                            let transactions = locked_manager
                                .get_transactions_of_account(account_id, timespan)
                                .await
                                .unwrap();
                            let accounts = locked_manager.get_accounts_hash_map().await.unwrap();
                            let mut transaction_tuples = Vec::with_capacity(transactions.len());
                            for transaction in transactions {
                                let source = accounts.get(transaction.source()).unwrap().clone();
                                let destination =
                                    accounts.get(transaction.destination()).unwrap().clone();
                                transaction_tuples.push((transaction, source, destination));
                            }
                            transaction_tuples
                        },
                        |x| AppMessage::ViewAccountMessage(Message::SetTransactions(x)),
                    ),
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        match &self.account {
            fm_core::account::Account::AssetAccount(acc) => {
                asset_account_view(acc, &self.transactions, &self.current_value)
            }
            _ => widget::text("comming soon").into(),
        }
    }
}

fn asset_account_view<'a>(
    account: &'a fm_core::account::AssetAccount,
    transactions: &'a [(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )],
    current_value: &fm_core::Currency,
) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
    widget::column![
        widget::row![
            widget::column![
                widget::text(format!("Account: {}", account.name())),
                widget::text(format!("Notes: {}", account.note().unwrap_or(""))),
                widget::text(format!("IBAN: {}", account.iban().unwrap_or(""))),
                widget::text(format!("BIC/Swift: {}", account.bic().unwrap_or(""))),
                widget::row![
                    widget::text("Current Amount: "),
                    utils::colored_currency_display(current_value)
                ],
            ],
            widget::Space::with_width(iced::Length::Fill),
            widget::button("Edit").on_press(Message::Edit),
        ],
        widget::horizontal_rule(10),
        super::super::timespan_input::TimespanInput::new(Message::ChangeTransactionTimespan)
            .into_element(),
        utils::transaction_table(
            transactions.to_vec(),
            |transaction| Some(*transaction.destination() == account.id()),
            Message::ViewTransaction,
            Message::ViewAccount,
        )
    ]
    .into()
}
