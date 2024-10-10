use super::{super::utils, category};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum AccountType {
    AssetAccount,
    BookCheckingAccount,
}

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    EditAssetAccount(fm_core::account::AssetAccount),
    EditBookCheckingAccount(fm_core::account::BookCheckingAccount),
    Task(iced::Task<Message>),
    AccountDeleted(AccountType),
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
        Vec<fm_core::Category>,
    ),
    Delete,
    Deleted(Arc<std::result::Result<(), fm_core::DeleteAccountError>>),
    TransactionTableMessage(utils::transaction_table::Message),
}

#[derive(Debug)]
pub enum Account {
    NotLoaded,
    Loaded {
        account: fm_core::account::Account,
        current_value: fm_core::Currency,
        transaction_table: utils::TransactionTable,
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
        categories: Vec<fm_core::Category>,
    ) -> Self {
        let account_id = *account.id();
        Self::Loaded {
            current_value: account_sum,
            account,
            transaction_table: utils::TransactionTable::new(
                transactions,
                categories,
                move |transaction| Some(*transaction.destination() == account_id),
            ),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
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
                    .get_account_sum(&account, time::OffsetDateTime::now_utc())
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
                let categories = locked_manager.get_categories().await.unwrap();
                Message::Initialize(account, account_sum, transaction_tuples, categories)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize(account, current_value, transactions, categories) => {
                let account_id = *account.id();
                *self = Self::Loaded {
                    account,
                    current_value,
                    transaction_table: utils::TransactionTable::new(
                        transactions,
                        categories,
                        move |transaction| Some(*transaction.destination() == account_id),
                    ),
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
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    transaction_table.change_transactions(new_transactions);
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
            Message::Delete => {
                if let Self::Loaded { account, .. } = self {
                    if let rfd::MessageDialogResult::No = rfd::MessageDialog::new()
                        .set_title("Delete Account?")
                        .set_description(format!(
                            "Do you really want to delete the account {}?",
                            account.name()
                        ))
                        .set_level(rfd::MessageLevel::Warning)
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        return Action::None;
                    }

                    let acc_id = *account.id();
                    Action::Task(iced::Task::future(async move {
                        let mut locked_manager = finance_manager.lock().await;
                        let result = locked_manager.delete_account(acc_id, false).await;
                        Message::Deleted(Arc::new(result))
                    }))
                } else {
                    Action::None
                }
            }
            Message::Deleted(result) => match &*result {
                Ok(_) => {
                    if let Self::Loaded { account, .. } = self {
                        match account {
                            fm_core::account::Account::AssetAccount(_) => {
                                return Action::AccountDeleted(AccountType::AssetAccount)
                            }
                            fm_core::account::Account::BookCheckingAccount(_) => {
                                return Action::AccountDeleted(AccountType::BookCheckingAccount)
                            }
                        }
                    }
                    Action::None
                }
                Err(e) => match e {
                    fm_core::DeleteAccountError::RelatedTransactionsExist => {
                        rfd::MessageDialog::new()
                            .set_title("Error")
                            .set_description("Related transactions exist")
                            .set_level(rfd::MessageLevel::Error)
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();
                        Action::None
                    }
                    fm_core::DeleteAccountError::Other(e) => {
                        todo!("Handle error: {:?}", e);
                    }
                },
            },
            Message::TransactionTableMessage(msg) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    return match transaction_table.update(msg, finance_manager) {
                        utils::transaction_table::Action::None => Action::None,
                        utils::transaction_table::Action::ViewTransaction(id) => {
                            Action::ViewTransaction(id)
                        }
                        utils::transaction_table::Action::ViewAccount(id) => {
                            Action::ViewAccount(id)
                        }
                        utils::transaction_table::Action::Task(task) => {
                            Action::Task(task.map(Message::TransactionTableMessage))
                        }
                    };
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if let Self::Loaded {
            account,
            transaction_table,
            current_value,
        } = self
        {
            match account {
                fm_core::account::Account::AssetAccount(acc) => {
                    asset_account_view(acc, transaction_table, current_value)
                }
                fm_core::account::Account::BookCheckingAccount(acc) => {
                    book_checking_account_view(acc, transaction_table, current_value)
                }
            }
        } else {
            widget::text!("Loading...").into()
        }
    }
}

fn asset_account_view<'a>(
    account: &'a fm_core::account::AssetAccount,
    transaction_table: &'a utils::TransactionTable,
    current_value: &fm_core::Currency,
) -> iced::Element<'a, Message> {
    widget::column![
        utils::heading("Asset Account", utils::HeadingLevel::H1),
        widget::row![
            widget::column![
                widget::text!("Account: {}", account.name()),
                widget::text!("Notes: {}", account.note().unwrap_or("")),
                widget::text!(
                    "IBAN: {}",
                    account
                        .iban()
                        .clone()
                        .map_or(String::new(), |iban| iban.to_string())
                ),
                widget::text!("BIC/Swift: {}", account.bic().unwrap_or("")),
                widget::text!("Offset: {}", account.offset()),
                widget::row![
                    widget::text("Current Amount: "),
                    utils::colored_currency_display(current_value)
                ],
            ],
            widget::Space::with_width(iced::Length::Fill),
            widget::column![
                widget::button("Edit").on_press(Message::Edit),
                widget::button("Delete")
                    .on_press(Message::Delete)
                    .style(widget::button::danger)
            ]
            .spacing(10)
        ],
        widget::horizontal_rule(10),
        utils::TimespanInput::new(Message::ChangeTransactionTimespan, None).into_element(),
        transaction_table
            .view()
            .map(Message::TransactionTableMessage),
    ]
    .height(iced::Fill)
    .into()
}

fn book_checking_account_view<'a>(
    account: &'a fm_core::account::BookCheckingAccount,
    transaction_table: &'a utils::TransactionTable,
    current_value: &fm_core::Currency,
) -> iced::Element<'a, Message> {
    widget::column![
        utils::heading("Book Checking Account", utils::HeadingLevel::H1),
        widget::row![
            widget::column![
                widget::text!("Account: {}", account.name()),
                widget::text!("Notes: {}", account.note().unwrap_or("")),
                widget::text!(
                    "IBAN: {}",
                    account
                        .iban()
                        .clone()
                        .map_or(String::new(), |iban| iban.to_string())
                ),
                widget::text!("BIC/Swift: {}", account.bic().unwrap_or("")),
                widget::row![
                    widget::text("Current Amount: "),
                    utils::colored_currency_display(current_value)
                ],
            ],
            widget::Space::with_width(iced::Length::Fill),
            widget::column![
                widget::button("Edit").on_press(Message::Edit),
                widget::button("Delete")
                    .on_press(Message::Delete)
                    .style(widget::button::danger)
            ]
            .spacing(10)
        ],
        widget::horizontal_rule(10),
        utils::TimespanInput::new(Message::ChangeTransactionTimespan, None).into_element(),
        transaction_table
            .view()
            .map(Message::TransactionTableMessage),
    ]
    .height(iced::Fill)
    .into()
}
