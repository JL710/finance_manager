use super::super::utils;
use async_std::sync::Mutex;
use fm_core::transaction_filter::TransactionFilter;
use std::sync::Arc;

pub enum Action {
    None,
    Task(iced::Task<Message>),
    ViewAccount(fm_core::Id),
    ViewTransaction(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFilter(TransactionFilter),
    ToggleEditFilter,
    ViewAccount(fm_core::Id),
    ViewTransaction(fm_core::Id),
    UpdateTransactions(
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
    Initialize {
        accounts: Vec<fm_core::account::Account>,
        categories: Vec<fm_core::Category>,
    },
    TransactionTableMessage(utils::transaction_table::Message),
}

#[derive(Debug)]
pub struct FilterTransactionView {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    change_filter: bool,
    transaction_table: utils::TransactionTable,
    sums: Vec<(fm_core::DateTime, fm_core::Currency)>,
    filter: TransactionFilter,
}

impl FilterTransactionView {
    pub fn new(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                accounts: Vec::new(),
                categories: Vec::new(),
                change_filter: false,
                transaction_table: utils::TransactionTable::new(Vec::new(), Vec::new(), |_| None),
                sums: Vec::new(),
                filter: TransactionFilter::default(),
            },
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let accounts = locked_manager.get_accounts().await.unwrap();
                let categories = locked_manager.get_categories().await.unwrap();
                Message::Initialize {
                    accounts,
                    categories,
                }
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize {
                accounts,
                categories,
            } => {
                self.accounts = accounts;
                self.categories = categories;
                self.transaction_table =
                    utils::TransactionTable::new(Vec::new(), self.categories.clone(), |_| None);
            }
            Message::ToggleEditFilter => {
                self.change_filter = !self.change_filter;
            }
            Message::ChangeFilter(filter) => {
                self.filter = filter.clone();
                self.change_filter = false;
                return Action::Task(iced::Task::future(async move {
                    let locked_manager = finance_manager.lock().await;
                    let transactions = locked_manager
                        .get_filtered_transactions(filter.clone())
                        .await
                        .unwrap();
                    let accounts = locked_manager.get_accounts().await.unwrap();

                    let mut tuples = Vec::new();
                    for transaction in transactions {
                        let source = accounts
                            .iter()
                            .find(|x| x.id() == transaction.source())
                            .unwrap()
                            .clone();
                        let destination = accounts
                            .iter()
                            .find(|x| x.id() == transaction.destination())
                            .unwrap()
                            .clone();
                        tuples.push((transaction, source, destination));
                    }
                    Message::UpdateTransactions(tuples)
                }));
            }
            Message::ViewAccount(id) => return Action::ViewAccount(id),
            Message::ViewTransaction(id) => return Action::ViewTransaction(id),
            Message::UpdateTransactions(transactions) => {
                self.sums = fm_core::sum_up_transactions_by_day(
                    transactions.clone().into_iter().map(|x| x.0).collect(),
                    |_| fm_core::Sign::Positive,
                );
                self.transaction_table.change_transactions(transactions);
            }
            Message::TransactionTableMessage(msg) => {
                match self.transaction_table.update(msg, finance_manager) {
                    utils::transaction_table::Action::None => return Action::None,
                    utils::transaction_table::Action::ViewTransaction(id) => {
                        return Action::ViewTransaction(id)
                    }
                    utils::transaction_table::Action::ViewAccount(id) => {
                        return Action::ViewAccount(id)
                    }
                    utils::transaction_table::Action::Task(task) => {
                        return Action::Task(task.map(Message::TransactionTableMessage))
                    }
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            utils::heading("Find Transactions", utils::HeadingLevel::H1),
            iced::widget::text("Filter Transactions"),
            iced::widget::row![
                iced::widget::text("Total: "),
                iced::widget::text!(
                    "{}",
                    self.sums
                        .last()
                        .map_or(fm_core::Currency::default(), |x| x.1.clone())
                )
            ],
            iced::widget::button(iced::widget::text("Edit Filter"))
                .on_press(Message::ToggleEditFilter),
            if self.change_filter {
                utils::FilterComponent::new(
                    self.filter.clone(),
                    Message::ChangeFilter,
                    &self.accounts,
                    &self.categories,
                )
                .into_element()
            } else {
                self.transaction_table
                    .view()
                    .map(Message::TransactionTableMessage)
            }
        ]
        .spacing(10)
        .height(iced::Fill)
        .width(iced::Fill)
        .into()
    }
}
