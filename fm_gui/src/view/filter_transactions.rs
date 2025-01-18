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
    FilterComponent(Box<utils::filter_component::InnerMessage>),
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
        bills: Vec<fm_core::Bill>,
        budgets: Vec<fm_core::Budget>,
    },
    TransactionTable(utils::transaction_table::Message),
}

#[derive(Debug)]
pub struct FilterTransactionView {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    bills: Vec<fm_core::Bill>,
    budgets: Vec<fm_core::Budget>,
    change_filter: Option<utils::filter_component::FilterComponent>,
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
                bills: Vec::new(),
                budgets: Vec::new(),
                change_filter: None,
                transaction_table: utils::TransactionTable::new(Vec::new(), Vec::new(), |_| None),
                sums: Vec::new(),
                filter: TransactionFilter::default(),
            },
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let accounts = locked_manager.get_accounts().await.unwrap();
                let categories = locked_manager.get_categories().await.unwrap();
                let bills = locked_manager.get_bills().await.unwrap();
                let budgets = locked_manager.get_budgets().await.unwrap();
                Message::Initialize {
                    accounts,
                    categories,
                    bills,
                    budgets,
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
                bills,
                budgets,
            } => {
                self.accounts = accounts;
                self.categories = categories;
                self.bills = bills;
                self.budgets = budgets;
                self.transaction_table =
                    utils::TransactionTable::new(Vec::new(), self.categories.clone(), |_| None);
            }
            Message::ToggleEditFilter => {
                self.change_filter = if self.change_filter.is_some() {
                    None
                } else {
                    Some(
                        utils::filter_component::FilterComponent::new(
                            self.accounts.clone(),
                            self.categories.clone(),
                            self.bills.clone(),
                            self.budgets.clone(),
                        )
                        .with_filter(self.filter.clone()),
                    )
                };
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
            Message::TransactionTable(msg) => {
                match self.transaction_table.update(msg, finance_manager) {
                    utils::transaction_table::Action::None => return Action::None,
                    utils::transaction_table::Action::ViewTransaction(id) => {
                        return Action::ViewTransaction(id)
                    }
                    utils::transaction_table::Action::ViewAccount(id) => {
                        return Action::ViewAccount(id)
                    }
                    utils::transaction_table::Action::Task(task) => {
                        return Action::Task(task.map(Message::TransactionTable))
                    }
                }
            }
            Message::FilterComponent(m) => {
                if let Some(component) = &mut self.change_filter {
                    match component.update(*m) {
                        utils::filter_component::Action::Submit(new_filter) => {
                            self.filter = new_filter.clone();
                            self.change_filter = None;
                            return Action::Task(iced::Task::future(async move {
                                let locked_manager = finance_manager.lock().await;
                                let transactions = locked_manager
                                    .get_filtered_transactions(new_filter.clone())
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
                        utils::filter_component::Action::None => {}
                    }
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Find Transactions",
            iced::widget::column![
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
                if let Some(filter_component) = &self.change_filter {
                    filter_component
                        .view()
                        .map(|x| Message::FilterComponent(Box::new(x)))
                } else {
                    self.transaction_table.view().map(Message::TransactionTable)
                }
            ]
            .spacing(10)
            .height(iced::Fill)
            .width(iced::Fill),
        )
    }
}
