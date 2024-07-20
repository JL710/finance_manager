use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    account_id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    let (view, task) = Account::fetch(finance_manager.clone(), account_id);
    iced::Task::done(AppMessage::SwitchView(View::ViewAccount(view)))
        .chain(task.map(AppMessage::ViewAccountMessage))
}

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    EditAssetAccount(fm_core::account::AssetAccount),
    EditBookCheckingAccount(fm_core::account::BookCheckingAccount),
    Task(iced::Task<Message>),
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
    Initialize(
        fm_core::account::Account,
        fm_core::Currency,
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
}

#[derive(Debug, Clone)]
pub enum Account {
    NotLoaded,
    Loaded {
        account: fm_core::account::Account,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        current_value: fm_core::Currency,
    },
}

impl Account {
    pub fn new(
        account: fm_core::account::Account,
        account_sum: fm_core::Currency,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ) -> Self {
        Self::Loaded {
            current_value: account_sum,
            account,
            transactions,
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        account_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            iced::Task::future(async move {
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
                let transactions = locked_manager
                    .get_transactions_of_account(*account.id(), (None, None))
                    .await
                    .unwrap();
                let accounts = locked_manager.get_accounts_hash_map().await.unwrap();
                let mut transaction_tuples = Vec::with_capacity(transactions.len());
                for transaction in transactions {
                    let source = accounts.get(transaction.source()).unwrap().clone();
                    let destination = accounts.get(transaction.destination()).unwrap().clone();
                    transaction_tuples.push((transaction, source, destination));
                }
                Message::Initialize(account, account_sum, transaction_tuples)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::Initialize(account, current_value, transactions) => {
                *self = Self::Loaded {
                    account,
                    current_value,
                    transactions,
                };
                Action::None
            }
            Message::Edit => {
                if let Self::Loaded { account, .. } = self {
                    match account {
                        fm_core::account::Account::AssetAccount(acc) => {
                            Action::EditAssetAccount(acc.clone())
                        }
                        fm_core::account::Account::BookCheckingAccount(acc) => {
                            Action::EditBookCheckingAccount(acc.clone())
                        }
                    }
                } else {
                    Action::None
                }
            }
            Message::ViewTransaction(id) => Action::ViewTransaction(id),
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::SetTransactions(new_transactions) => {
                if let Self::Loaded { transactions, .. } = self {
                    *transactions = new_transactions;
                }
                Action::None
            }
            Message::ChangeTransactionTimespan(timespan) => {
                let account_id = if let Self::Loaded { account, .. } = self {
                    *account.id()
                } else {
                    return Action::None;
                };
                Action::Task(iced::Task::future(async move {
                    let locked_manager = finance_manager.lock().await;
                    let transactions = locked_manager
                        .get_transactions_of_account(account_id, timespan)
                        .await
                        .unwrap();
                    let accounts = locked_manager.get_accounts_hash_map().await.unwrap();
                    let mut transaction_tuples = Vec::with_capacity(transactions.len());
                    for transaction in transactions {
                        let source = accounts.get(transaction.source()).unwrap().clone();
                        let destination = accounts.get(transaction.destination()).unwrap().clone();
                        transaction_tuples.push((transaction, source, destination));
                    }
                    Message::SetTransactions(transaction_tuples)
                }))
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if let Self::Loaded {
            account,
            transactions,
            current_value,
        } = self
        {
            match account {
                fm_core::account::Account::AssetAccount(acc) => {
                    asset_account_view(acc, transactions, current_value)
                }
                fm_core::account::Account::BookCheckingAccount(acc) => {
                    book_checking_account_view(acc, transactions, current_value)
                }
            }
        } else {
            widget::text!("Loading...").into()
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
) -> iced::Element<'a, Message> {
    widget::column![
        utils::heading("Asset Account", utils::HeadingLevel::H1),
        widget::row![
            widget::column![
                widget::text!("Account: {}", account.name()),
                widget::text!("Notes: {}", account.note().unwrap_or("")),
                widget::text!("IBAN: {}", account.iban().unwrap_or("")),
                widget::text!("BIC/Swift: {}", account.bic().unwrap_or("")),
                widget::text!("Offset: {}", account.offset()),
                widget::row![
                    widget::text("Current Amount: "),
                    utils::colored_currency_display(current_value)
                ],
            ],
            widget::Space::with_width(iced::Length::Fill),
            widget::button("Edit").on_press(Message::Edit),
        ],
        widget::horizontal_rule(10),
        utils::TimespanInput::new(Message::ChangeTransactionTimespan, None).into_element(),
        utils::transaction_table(
            transactions.to_vec(),
            |transaction| Some(*transaction.destination() == account.id()),
            Message::ViewTransaction,
            Message::ViewAccount,
        )
    ]
    .into()
}

fn book_checking_account_view<'a>(
    account: &'a fm_core::account::BookCheckingAccount,
    transactions: &'a [(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )],
    current_value: &fm_core::Currency,
) -> iced::Element<'a, Message> {
    widget::column![
        utils::heading("Book Checking Account", utils::HeadingLevel::H1),
        widget::row![
            widget::column![
                widget::text!("Account: {}", account.name()),
                widget::text!("Notes: {}", account.note().unwrap_or("")),
                widget::text!("IBAN: {}", account.iban().unwrap_or("")),
                widget::text!("BIC/Swift: {}", account.bic().unwrap_or("")),
                widget::row![
                    widget::text("Current Amount: "),
                    utils::colored_currency_display(current_value)
                ],
            ],
            widget::Space::with_width(iced::Length::Fill),
            widget::button("Edit").on_press(Message::Edit),
        ],
        widget::horizontal_rule(10),
        utils::TimespanInput::new(Message::ChangeTransactionTimespan, None).into_element(),
        utils::transaction_table(
            transactions.to_vec(),
            |transaction| Some(*transaction.destination() == account.id()),
            Message::ViewTransaction,
            Message::ViewAccount,
        )
    ]
    .into()
}
