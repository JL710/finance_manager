use anyhow::Context;
use iced::widget;
use std::sync::Arc;
use utils::date_time::date_span_input;

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
    Task(iced::Task<MessageContainer>),
    AccountDeleted(AccountType),
}

#[derive(Debug, Clone)]
struct Init {
    account: fm_core::account::Account,
    value: fm_core::Currency,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    categories: Vec<fm_core::Category>,
    budgets: Vec<fm_core::Budget>,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    Edit,
    ChangeTransactionTimespan(date_span_input::Action),
    SetTransactions(
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
    Initialize(Box<Init>),
    Delete,
    Deleted(Arc<std::result::Result<(), fm_core::DeleteAccountError>>),
    TransactionTable(utils::transaction_table::Message),
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum View {
    NotLoaded,
    Loaded {
        account: fm_core::account::Account,
        current_value: fm_core::Currency,
        transaction_table: utils::TransactionTable,
        timespan_input: date_span_input::State,
    },
}

impl View {
    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        account_id: fm_core::Id,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::NotLoaded,
            utils::failing_task(async move {
                let account = finance_controller
                    .get_account(account_id)
                    .await?
                    .context("Account could not be found")?;
                let account_sum = finance_controller
                    .get_account_sum(&account, time::OffsetDateTime::now_utc())
                    .await?;
                let transactions = finance_controller
                    .get_transactions_of_account(*account.id(), (None, None))
                    .await?;
                let accounts = finance_controller.get_accounts_hash_map().await?;
                let mut transaction_tuples = Vec::with_capacity(transactions.len());
                for transaction in transactions {
                    let source = accounts
                        .get(transaction.source())
                        .context(format!("Could not find account {}", transaction.source()))?
                        .clone();
                    let destination = accounts
                        .get(transaction.destination())
                        .context(format!(
                            "Could not find account {}",
                            transaction.destination()
                        ))?
                        .clone();
                    transaction_tuples.push((transaction, source, destination));
                }
                let categories = finance_controller.get_categories().await?;
                let budgets = finance_controller.get_budgets().await?;
                Ok(Message::Initialize(Box::new(Init {
                    account,
                    value: account_sum,
                    transactions: transaction_tuples,
                    categories,
                    budgets,
                })))
            })
            .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message.0 {
            Message::Initialize(init) => {
                let account_id = *init.account.id();
                *self = Self::Loaded {
                    account: init.account,
                    current_value: init.value,
                    transaction_table: utils::TransactionTable::new(
                        init.transactions,
                        init.categories,
                        init.budgets,
                        move |transaction| Some(*transaction.destination() == account_id),
                    ),
                    timespan_input: date_span_input::State::default(),
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
            Message::SetTransactions(new_transactions) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    transaction_table.change_transactions(new_transactions);
                }
                Action::None
            }
            Message::ChangeTransactionTimespan(action) => {
                let timespan = if let Self::Loaded { timespan_input, .. } = self {
                    timespan_input.perform(action);
                    timespan_input.timespan()
                } else {
                    return Action::None;
                };
                let account_id = if let Self::Loaded { account, .. } = self {
                    *account.id()
                } else {
                    return Action::None;
                };
                Action::Task(
                    utils::failing_task(async move {
                        let transactions = finance_controller
                            .get_transactions_of_account(account_id, timespan)
                            .await
                            .context(format!(
                                "Error while fetching transactions of account {}.",
                                account_id
                            ))?;
                        let accounts = finance_controller
                            .get_accounts_hash_map()
                            .await
                            .context("Error while fetching accounts")?;
                        let mut transaction_tuples = Vec::with_capacity(transactions.len());
                        for transaction in transactions {
                            let source = accounts
                                .get(transaction.source())
                                .context(format!(
                                    "Could not find account {}",
                                    transaction.source()
                                ))?
                                .clone();
                            let destination = accounts
                                .get(transaction.destination())
                                .context(format!(
                                    "Could not find account {}",
                                    transaction.destination()
                                ))?
                                .clone();
                            transaction_tuples.push((transaction, source, destination));
                        }
                        Ok(Message::SetTransactions(transaction_tuples))
                    })
                    .map(MessageContainer),
                )
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
                    Action::Task(
                        iced::Task::future(async move {
                            let result = finance_controller.delete_account(acc_id, false).await;
                            Message::Deleted(Arc::new(result))
                        })
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
            }
            Message::Deleted(result) => match &*result {
                Ok(_) => {
                    if let Self::Loaded { account, .. } = self {
                        match account {
                            fm_core::account::Account::AssetAccount(_) => {
                                return Action::AccountDeleted(AccountType::AssetAccount);
                            }
                            fm_core::account::Account::BookCheckingAccount(_) => {
                                return Action::AccountDeleted(AccountType::BookCheckingAccount);
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
            Message::TransactionTable(msg) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    return match transaction_table.update(msg, finance_controller) {
                        utils::transaction_table::Action::None => Action::None,
                        utils::transaction_table::Action::ViewTransaction(id) => {
                            Action::ViewTransaction(id)
                        }
                        utils::transaction_table::Action::ViewAccount(id) => {
                            Action::ViewAccount(id)
                        }
                        utils::transaction_table::Action::Task(task) => {
                            Action::Task(task.map(Message::TransactionTable).map(MessageContainer))
                        }
                    };
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        if let Self::Loaded {
            account,
            transaction_table,
            current_value,
            timespan_input,
        } = self
        {
            match account {
                fm_core::account::Account::AssetAccount(acc) => {
                    asset_account_view(acc, transaction_table, current_value, timespan_input)
                        .map(MessageContainer)
                }
                fm_core::account::Account::BookCheckingAccount(acc) => book_checking_account_view(
                    acc,
                    transaction_table,
                    current_value,
                    timespan_input,
                )
                .map(MessageContainer),
            }
        } else {
            super::view("Account", "Loading")
        }
    }
}

fn asset_account_view<'a>(
    account: &'a fm_core::account::AssetAccount,
    transaction_table: &'a utils::TransactionTable,
    current_value: &fm_core::Currency,
    timespan_input: &'a date_span_input::State,
) -> iced::Element<'a, Message> {
    super::view(
        "Asset Account",
        utils::spaced_column![
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
                utils::spaced_column![
                    utils::button::edit(Some(Message::Edit)),
                    utils::button::delete(Some(Message::Delete))
                ]
            ],
            widget::horizontal_rule(10),
            date_span_input::date_span_input(timespan_input)
                .view()
                .map(Message::ChangeTransactionTimespan),
            transaction_table.view().map(Message::TransactionTable),
        ]
        .height(iced::Fill),
    )
}

fn book_checking_account_view<'a>(
    account: &'a fm_core::account::BookCheckingAccount,
    transaction_table: &'a utils::TransactionTable,
    current_value: &fm_core::Currency,
    timespan_input: &'a date_span_input::State,
) -> iced::Element<'a, Message> {
    super::view(
        "Book Checking Account",
        utils::spaced_column![
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
                utils::spaced_column![
                    utils::button::edit(Some(Message::Edit)),
                    utils::button::delete(Some(Message::Delete))
                ]
            ],
            widget::horizontal_rule(10),
            date_span_input::date_span_input(timespan_input)
                .view()
                .map(Message::ChangeTransactionTimespan),
            transaction_table.view().map(Message::TransactionTable),
        ]
        .height(iced::Fill),
    )
}
