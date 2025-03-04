use anyhow::Context;
use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;
use utils::date_time::date_time_input;

pub enum Action {
    None,
    BillCreated(fm_core::Id),
    Cancel,
    CancelWithId(fm_core::Id),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    DueDateChanged(date_time_input::Action),
    NameInputChanged(String),
    ValueChanged(utils::currency_input::Action),
    DescriptionInputChanged(widget::text_editor::Action),
    AddTransactionToggle,
    ChangeTransactionSign(fm_core::Id, fm_core::Sign),
    RemoveTransaction(fm_core::Id),
    AddTransaction(add_transaction::Message),
    Submit,
    Initialize(
        Option<fm_core::Bill>,
        Vec<(fm_core::Transaction, fm_core::Sign)>,
        Vec<fm_core::account::Account>,
    ),
    BillCreated(fm_core::Id),
    TransactionTable(utils::table_view::InnerMessage<Message>),
    Cancel,
}

#[derive(Debug)]
pub struct View {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: widget::text_editor::Content,
    value: utils::currency_input::State,
    due_date_input: date_time_input::State,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
    transaction_table: utils::table_view::State<
        (fm_core::Transaction, fm_core::Sign),
        Vec<fm_core::account::Account>,
    >,
    add_transaction: Option<add_transaction::AddTransaction>,
    submitted: bool,
}

impl View {
    /// This can not work standalone! Only use as pore fetch helper.
    fn default() -> Self {
        Self {
            id: None,
            name_input: String::new(),
            description_input: widget::text_editor::Content::default(),
            value: utils::currency_input::State::default(),
            due_date_input: date_time_input::State::default(),
            transactions: Vec::new(),
            transaction_table: utils::table_view::State::new(Vec::new(), Vec::new())
                .sort_by(
                    |a: &(fm_core::Transaction, fm_core::Sign),
                     b: &(fm_core::Transaction, fm_core::Sign),
                     column| match column {
                        0 => match (a.1, b.1) {
                            (fm_core::Sign::Positive, fm_core::Sign::Negative) => {
                                std::cmp::Ordering::Less
                            }
                            (fm_core::Sign::Negative, fm_core::Sign::Positive) => {
                                std::cmp::Ordering::Greater
                            }
                            _ => std::cmp::Ordering::Equal,
                        },
                        2 => a.0.title().cmp(b.0.title()),
                        3 => a.0.amount().cmp(&b.0.amount()),
                        4 => a.0.date().cmp(b.0.date()),
                        5 => a.0.source().cmp(b.0.source()),
                        6 => a.0.destination().cmp(b.0.destination()),
                        _ => panic!(),
                    },
                )
                .sortable_columns([0, 2, 3, 4, 5, 6]),
            add_transaction: None,
            submitted: false,
        }
    }

    pub fn new_with_transaction(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
        transaction: fm_core::Transaction,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            utils::failing_task(async move {
                let locked_manager = finance_manager.lock().await;

                let accounts = locked_manager.get_accounts().await?;

                Ok(Message::Initialize(
                    None,
                    vec![(transaction, fm_core::Sign::Negative)],
                    accounts,
                ))
            }),
        )
    }

    pub fn fetch(
        id: Option<fm_core::Id>,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                id: None,
                name_input: String::new(),
                description_input: widget::text_editor::Content::default(),
                value: utils::currency_input::State::default(),
                due_date_input: date_time_input::State::default(),
                transactions: Vec::new(),
                transaction_table: utils::table_view::State::new(Vec::new(), Vec::new())
                    .sort_by(
                        |a: &(fm_core::Transaction, fm_core::Sign),
                         b: &(fm_core::Transaction, fm_core::Sign),
                         column| match column {
                            0 => match (a.1, b.1) {
                                (fm_core::Sign::Positive, fm_core::Sign::Negative) => {
                                    std::cmp::Ordering::Less
                                }
                                (fm_core::Sign::Negative, fm_core::Sign::Positive) => {
                                    std::cmp::Ordering::Greater
                                }
                                _ => std::cmp::Ordering::Equal,
                            },
                            2 => a.0.title().cmp(b.0.title()),
                            3 => a.0.amount().cmp(&b.0.amount()),
                            4 => a.0.date().cmp(b.0.date()),
                            5 => a.0.source().cmp(b.0.source()),
                            6 => a.0.destination().cmp(b.0.destination()),
                            _ => panic!(),
                        },
                    )
                    .sortable_columns([0, 2, 3, 4, 5, 6]),
                add_transaction: None,
                submitted: false,
            },
            utils::failing_task(async move {
                let locked_manager = finance_manager.lock().await;

                let bill = if let Some(id) = id {
                    Some(
                        locked_manager
                            .get_bill(&id)
                            .await?
                            .context("Could not find bill")?,
                    )
                } else {
                    None
                };

                let mut transactions = Vec::new();

                if let Some(existing_bill) = &bill {
                    for (transaction_id, sign) in existing_bill.transactions() {
                        transactions.push((
                            locked_manager
                                .get_transaction(*transaction_id)
                                .await?
                                .context("Could not find transaction")?,
                            *sign,
                        ));
                    }
                }

                let accounts = locked_manager.get_accounts().await?;

                Ok(Message::Initialize(bill, transactions, accounts))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Cancel => {
                if let Some(id) = self.id {
                    return Action::CancelWithId(id);
                } else {
                    return Action::Cancel;
                }
            }
            Message::BillCreated(id) => {
                return Action::BillCreated(id);
            }
            Message::Initialize(bill, transactions, accounts) => {
                if let Some(bill) = bill {
                    self.id = Some(*bill.id());
                    bill.name().clone_into(&mut self.name_input);
                    self.description_input = widget::text_editor::Content::with_text(
                        &bill.description().clone().unwrap_or_default(),
                    );
                    self.value = utils::currency_input::State::new(bill.value().clone());
                    self.due_date_input =
                        utils::date_time::date_time_input::State::new(*bill.due_date());
                }
                self.transactions = transactions.clone();
                self.transaction_table.set_items(transactions);
                self.transaction_table.set_context(accounts);
                self.add_transaction = None;
            }
            Message::DueDateChanged(action) => {
                self.due_date_input.perform(action);
            }
            Message::NameInputChanged(name) => {
                self.name_input = name;
            }
            Message::ValueChanged(action) => {
                self.value.perform(action);
            }
            Message::DescriptionInputChanged(action) => {
                self.description_input.perform(action);
            }
            Message::Submit => {
                if !self.submittable() {
                    panic!("Cant Submit!")
                }
                self.submitted = true;
                let id_option = self.id;
                let name = self.name_input.clone();
                let description = if self.description_input.text().trim().is_empty() {
                    None
                } else {
                    Some(self.description_input.text())
                };
                let due_date = self.due_date_input.datetime();
                let value = self.value.currency().unwrap();
                let mut transactions =
                    std::collections::HashMap::with_capacity(self.transactions.len());
                for transaction in &self.transactions {
                    transactions.insert(*transaction.0.id(), transaction.1);
                }
                if let Some(id) = id_option {
                    return Action::Task(utils::failing_task(async move {
                        finance_manager
                            .lock()
                            .await
                            .update_bill(id, name, description, value, transactions, due_date)?
                            .await?;
                        Ok(Message::BillCreated(id))
                    }));
                } else {
                    return Action::Task(utils::failing_task(async move {
                        let mut locked_manager = finance_manager.lock().await;
                        let bill = locked_manager
                            .create_bill(name, description, value, transactions, due_date)?
                            .await?;
                        Ok(Message::BillCreated(*bill.id()))
                    }));
                }
            }
            Message::AddTransactionToggle => {
                if self.add_transaction.is_some() {
                    self.add_transaction = None;
                    return Action::None;
                }
                let ignored_transactions = self.transactions.iter().map(|x| *x.0.id()).collect();
                let (view, task) =
                    add_transaction::AddTransaction::fetch(finance_manager, ignored_transactions);
                self.add_transaction = Some(view);
                return Action::Task(task.map(Message::AddTransaction));
            }
            Message::ChangeTransactionSign(transaction_id, sign) => {
                self.transactions // FIXME: should the transactions only be stored in the transaction table state?
                    .iter_mut()
                    .find(|(x, _)| x.id() == &transaction_id)
                    .unwrap()
                    .1 = sign;
                self.transaction_table.set_items(self.transactions.clone());
            }
            Message::RemoveTransaction(transaction_id) => {
                self.transactions.retain(|x| *x.0.id() != transaction_id);
                self.transaction_table.set_items(self.transactions.clone());
            }
            Message::AddTransaction(m) => {
                if let Some(add_transaction) = &mut self.add_transaction {
                    match add_transaction.update(m, finance_manager) {
                        add_transaction::Action::Escape => {
                            self.add_transaction = None;
                        }
                        add_transaction::Action::AddTransactions(transactions) => {
                            for transaction in transactions {
                                self.transactions
                                    .push((transaction, fm_core::Sign::Negative));
                            }
                            self.transaction_table.set_items(self.transactions.clone());
                        }
                        add_transaction::Action::Task(task) => {
                            return Action::Task(task.map(Message::AddTransaction));
                        }
                        add_transaction::Action::None => {}
                    }
                }
            }
            Message::TransactionTable(inner) => match self.transaction_table.perform(inner) {
                utils::table_view::Action::OuterMessage(m) => {
                    return self.update(m, finance_manager);
                }
                utils::table_view::Action::Task(task) => {
                    return Action::Task(task.map(Message::TransactionTable));
                }
                _ => {}
            },
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        if let Some(add_transaction) = &self.add_transaction {
            return add_transaction.view().map(Message::AddTransaction);
        }

        super::view(
            "Create Bill",
            utils::spaced_column![
                utils::labeled_entry("Name", &self.name_input, Message::NameInputChanged, true),
                utils::spaced_row![
                    "Description: ",
                    widget::container(widget::scrollable(
                        widget::text_editor(&self.description_input)
                            .on_action(Message::DescriptionInputChanged)
                    ))
                    .max_height(200)
                ],
                utils::spal_row![
                    "Value: ",
                    utils::currency_input::currency_input(&self.value, true)
                        .view()
                        .map(Message::ValueChanged),
                ]
                .width(iced::Length::Fill),
                utils::spal_row![
                    "Due Date: ",
                    date_time_input::date_time_input(&self.due_date_input, false)
                        .view()
                        .map(Message::DueDateChanged),
                ]
                .width(iced::Length::Fill),
                "Transactions:",
                widget::container(
                    utils::table_view::table_view(&self.transaction_table)
                        .headers(["", "", "Title", "Amount", "Date", "Source", "Destination"])
                        .view(|(transaction, sign), accounts| {
                            let transaction_id = *transaction.id();
                            [
                                widget::checkbox("Positive", sign == &fm_core::Sign::Positive)
                                    .on_toggle(move |x| {
                                        Message::ChangeTransactionSign(
                                            transaction_id,
                                            if x {
                                                fm_core::Sign::Positive
                                            } else {
                                                fm_core::Sign::Negative
                                            },
                                        )
                                    })
                                    .into(),
                                utils::button::delete(Some(Message::RemoveTransaction(
                                    transaction_id,
                                ))),
                                widget::text(transaction.title().clone()).into(),
                                utils::colored_currency_display(
                                    &(if *sign == fm_core::Sign::Negative {
                                        transaction.amount().negative()
                                    } else {
                                        transaction.amount()
                                    }),
                                ),
                                widget::text(utils::date_time::to_date_string(*transaction.date()))
                                    .into(),
                                widget::text(
                                    accounts
                                        .iter()
                                        .find(|acc| acc.id() == transaction.source())
                                        .unwrap()
                                        .name(),
                                )
                                .into(),
                                widget::text(
                                    accounts
                                        .iter()
                                        .find(|acc| acc.id() == transaction.destination())
                                        .unwrap()
                                        .name(),
                                )
                                .into(),
                            ]
                        })
                        .map(Message::TransactionTable)
                )
                .height(iced::Fill),
                widget::button("Add Transaction").on_press(Message::AddTransactionToggle),
                utils::submit_cancel_row(
                    if self.submittable() {
                        Some(Message::Submit)
                    } else {
                        None
                    },
                    Some(Message::Cancel)
                ),
            ]
            .height(iced::Fill),
        )
    }

    fn submittable(&self) -> bool {
        !self.name_input.is_empty() && self.value.currency().is_some()
    }
}

mod add_transaction {
    use async_std::sync::Mutex;
    use std::sync::Arc;

    use iced::widget;

    pub enum Action {
        Escape,
        AddTransactions(Vec<fm_core::Transaction>),
        Task(iced::Task<Message>),
        None,
    }

    #[derive(Debug, Clone)]
    pub struct Init {
        accounts: Vec<fm_core::account::Account>,
        budgets: Vec<fm_core::Budget>,
        bills: Vec<fm_core::Bill>,
        categories: Vec<fm_core::Category>,
        transactions: Vec<fm_core::Transaction>,
        ignored_transactions: Vec<fm_core::Id>,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Back,
        FilterComponent(utils::filter_component::InnerMessage),
        AddTransaction(fm_core::Transaction),
        FetchedTransactions(Vec<fm_core::Transaction>),
        Table(utils::table_view::InnerMessage<Message>),
        Init(Init),
        AddAllTransactions,
    }

    #[derive(Debug)]
    pub struct AddTransaction {
        filter: Option<utils::filter_component::FilterComponent>,
        transactions: Vec<fm_core::Transaction>,
        ignored_transactions: Vec<fm_core::Id>,
        table: utils::table_view::State<fm_core::Transaction, Vec<fm_core::account::Account>>,
    }

    impl AddTransaction {
        pub fn new(
            filter: Option<utils::filter_component::FilterComponent>,
            transactions: Vec<fm_core::Transaction>,
            accounts: Vec<fm_core::account::Account>,
            ignored_transactions: Vec<fm_core::Id>,
        ) -> Self {
            Self {
                filter,
                transactions: transactions.clone(),
                ignored_transactions,
                table: utils::table_view::State::new(transactions, accounts)
                    .sort_by(|a, b, column| match column {
                        1 => a.title().cmp(b.title()),
                        2 => a.amount().cmp(&b.amount()),
                        3 => a.date().cmp(b.date()),
                        4 => a.source().cmp(b.source()),
                        5 => a.destination().cmp(b.destination()),
                        _ => panic!(),
                    })
                    .sortable_columns([1, 2, 3, 4, 5]),
            }
        }

        pub fn fetch(
            finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
            ignored_transactions: Vec<fm_core::Id>,
        ) -> (Self, iced::Task<Message>) {
            (
                Self::new(None, Vec::new(), Vec::new(), Vec::new()),
                utils::failing_task(async move {
                    let locked_manager = finance_manager.lock().await;
                    let accounts = locked_manager.get_accounts().await?;
                    let categories = locked_manager.get_categories().await?;
                    let bills = locked_manager.get_bills().await?;
                    let budgets = locked_manager.get_budgets().await?;
                    Ok(Message::Init(Init {
                        transactions: Vec::new(),
                        accounts,
                        categories,
                        bills,
                        budgets,
                        ignored_transactions,
                    }))
                }),
            )
        }

        pub fn update(
            &mut self,
            message: Message,
            finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
        ) -> Action {
            match message {
                Message::Back => Action::Escape,
                Message::FilterComponent(m) => {
                    if let Some(filter) = &mut self.filter {
                        match filter.update(m) {
                            utils::filter_component::Action::Submit(submitted_filter) => {
                                self.filter = None;
                                return Action::Task(utils::failing_task(async move {
                                    let locked_manager = finance_manager.lock().await;
                                    Ok(Message::FetchedTransactions(
                                        locked_manager
                                            .get_filtered_transactions(submitted_filter.clone())
                                            .await?,
                                    ))
                                }));
                            }
                            utils::filter_component::Action::None => {}
                        }
                    }
                    Action::None
                }
                Message::AddTransaction(transaction) => {
                    self.ignored_transactions.push(*transaction.id());
                    self.transactions.retain(|x| x.id() != transaction.id());
                    self.table.set_items(self.transactions.clone());
                    Action::AddTransactions(vec![transaction])
                }
                Message::AddAllTransactions => {
                    let mut transactions = Vec::new();
                    std::mem::swap(&mut self.transactions, &mut transactions);
                    self.ignored_transactions
                        .extend(transactions.iter().map(|x| *x.id()));
                    self.table.set_items(Vec::with_capacity(0));
                    Action::AddTransactions(transactions)
                }
                Message::FetchedTransactions(transactions) => {
                    self.transactions = transactions;
                    self.transactions.retain(|x| {
                        for id in &self.ignored_transactions {
                            if x.id() == id {
                                return false;
                            }
                        }
                        true
                    });
                    self.table.set_items(self.transactions.clone());
                    Action::None
                }
                Message::Table(inner) => match self.table.perform(inner) {
                    utils::table_view::Action::OuterMessage(m) => self.update(m, finance_manager),
                    utils::table_view::Action::Task(task) => Action::Task(task.map(Message::Table)),
                    _ => Action::None,
                },
                Message::Init(init) => {
                    self.transactions = init.transactions.clone();
                    self.table.set_items(init.transactions);
                    self.table.set_context(init.accounts.clone());
                    self.ignored_transactions = init.ignored_transactions;
                    self.filter = Some(utils::filter_component::FilterComponent::new(
                        init.accounts,
                        init.categories,
                        init.bills,
                        init.budgets,
                    ));
                    Action::None
                }
            }
        }

        pub fn view(&self) -> iced::Element<Message> {
            utils::spaced_column![
                utils::heading("Add", utils::HeadingLevel::H1),
                utils::spal_row![
                    widget::button("Back").on_press(Message::Back),
                    widget::horizontal_space(),
                    widget::button("Add All").on_press(Message::AddAllTransactions)
                ],
                if let Some(filter) = &self.filter {
                    iced::Element::new(
                        utils::spaced_column![
                            "Create Filter for Transactions:",
                            filter.view().map(Message::FilterComponent),
                        ]
                        .width(iced::Length::Fill),
                    )
                } else {
                    utils::table_view::table_view(&self.table)
                        .headers(["", "Title", "Amount", "Date", "Source", "Destination"])
                        .view(|x, accounts| {
                            [
                                utils::button::new("Add", Some(Message::AddTransaction(x.clone()))),
                                widget::text(x.title().clone()).into(),
                                widget::text(x.amount().to_num_string()).into(),
                                widget::text(utils::date_time::to_date_string(*x.date())).into(),
                                widget::text(
                                    accounts
                                        .iter()
                                        .find(|acc| acc.id() == x.source())
                                        .unwrap()
                                        .name(),
                                )
                                .into(),
                                widget::text(
                                    accounts
                                        .iter()
                                        .find(|acc| acc.id() == x.destination())
                                        .unwrap()
                                        .name(),
                                )
                                .into(),
                            ]
                        })
                        .map(Message::Table)
                }
            ]
            .height(iced::Fill)
            .width(iced::Fill)
            .into()
        }
    }
}
