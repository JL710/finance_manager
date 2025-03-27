use anyhow::Context;
use fm_core::transaction_filter::TransactionFilter;

pub enum Action {
    None,
    Task(iced::Task<Message>),
    ViewAccount(fm_core::Id),
    ViewTransaction(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    FilterComponent(Box<components::filter_component::InnerMessage>),
    ToggleEditFilter,
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
    TransactionTable(components::transaction_table::Message),
}

#[derive(Debug)]
pub struct View {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    bills: Vec<fm_core::Bill>,
    budgets: Vec<fm_core::Budget>,
    change_filter: Option<components::filter_component::FilterComponent>,
    transaction_table: components::TransactionTable,
    sums: Vec<(fm_core::DateTime, fm_core::Currency)>,
    filter: TransactionFilter,
}

impl View {
    pub fn new(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                accounts: Vec::new(),
                categories: Vec::new(),
                bills: Vec::new(),
                budgets: Vec::new(),
                change_filter: None,
                transaction_table: components::TransactionTable::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    |_| None,
                ),
                sums: Vec::new(),
                filter: TransactionFilter::default(),
            },
            components::failing_task(async move {
                let accounts = finance_controller.get_accounts().await?;
                let categories = finance_controller.get_categories().await?;
                let bills = finance_controller.get_bills().await?;
                let budgets = finance_controller.get_budgets().await?;
                Ok(Message::Initialize {
                    accounts,
                    categories,
                    bills,
                    budgets,
                })
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
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
                self.budgets = budgets.clone();
                self.transaction_table = components::TransactionTable::new(
                    Vec::new(),
                    self.categories.clone(),
                    budgets,
                    |_| None,
                );
            }
            Message::ToggleEditFilter => {
                self.change_filter = if self.change_filter.is_some() {
                    None
                } else {
                    Some(
                        components::filter_component::FilterComponent::new(
                            self.accounts.clone(),
                            self.categories.clone(),
                            self.bills.clone(),
                            self.budgets.clone(),
                        )
                        .with_filter(self.filter.clone()),
                    )
                };
            }
            Message::UpdateTransactions(transactions) => {
                self.sums = fm_core::sum_up_transactions_by_day(
                    transactions.clone().into_iter().map(|x| x.0).collect(),
                    |_| fm_core::Sign::Positive,
                );
                self.transaction_table.change_transactions(transactions);
            }
            Message::TransactionTable(msg) => {
                match self.transaction_table.update(msg, finance_controller) {
                    components::transaction_table::Action::None => return Action::None,
                    components::transaction_table::Action::ViewTransaction(id) => {
                        return Action::ViewTransaction(id);
                    }
                    components::transaction_table::Action::ViewAccount(id) => {
                        return Action::ViewAccount(id);
                    }
                    components::transaction_table::Action::Task(task) => {
                        return Action::Task(task.map(Message::TransactionTable));
                    }
                }
            }
            Message::FilterComponent(m) => {
                if let Some(component) = &mut self.change_filter {
                    match component.update(*m) {
                        components::filter_component::Action::Submit(new_filter) => {
                            self.filter = new_filter.clone();
                            self.change_filter = None;
                            return Action::Task(components::failing_task(async move {
                                let transactions = finance_controller
                                    .get_filtered_transactions(new_filter.clone())
                                    .await?;
                                let accounts = finance_controller.get_accounts().await?;

                                let mut tuples = Vec::new();
                                for transaction in transactions {
                                    let source = accounts
                                        .iter()
                                        .find(|x| *x.id() == transaction.source)
                                        .context(format!(
                                            "Could not find account {}",
                                            transaction.source
                                        ))?
                                        .clone();
                                    let destination = accounts
                                        .iter()
                                        .find(|x| *x.id() == transaction.destination)
                                        .context(format!(
                                            "Could not find account {}",
                                            transaction.destination
                                        ))?
                                        .clone();
                                    tuples.push((transaction, source, destination));
                                }
                                Ok(Message::UpdateTransactions(tuples))
                            }));
                        }
                        components::filter_component::Action::None => {}
                    }
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Find Transactions",
            components::spaced_column![
                "Filter Transactions",
                iced::widget::row![
                    "Total: ",
                    iced::widget::text!(
                        "{}",
                        self.sums
                            .last()
                            .map_or(fm_core::Currency::default(), |x| x.1.clone())
                    )
                ],
                components::button::edit_with_text("Edit Filter", Some(Message::ToggleEditFilter)),
                if let Some(filter_component) = &self.change_filter {
                    filter_component
                        .view()
                        .map(|x| Message::FilterComponent(Box::new(x)))
                } else {
                    self.transaction_table.view().map(Message::TransactionTable)
                }
            ]
            .height(iced::Fill)
            .width(iced::Fill),
        )
    }
}
