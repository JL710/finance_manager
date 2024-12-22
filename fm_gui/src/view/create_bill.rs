use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

pub enum Action {
    None,
    BillCreated(fm_core::Id),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    DueDateChanged(utils::date_input::Action),
    NameInputChanged(String),
    ValueChanged(Option<fm_core::Currency>),
    DescriptionInputChanged(widget::text_editor::Action),
    AddTransactionToggle,
    AddTransaction(fm_core::Transaction),
    ChangeTransactionSign(fm_core::Id, fm_core::Sign),
    RemoveTransaction(fm_core::Id),
    AddTransactionMessage(add_transaction::Message),
    FetchedAddTransaction(add_transaction::AddTransaction),
    Submit,
    Initialize(fm_core::Bill, Vec<(fm_core::Transaction, fm_core::Sign)>),
    BillCreated(fm_core::Id),
}

#[derive(Debug, Default)]
pub struct CreateBillView {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: widget::text_editor::Content,
    value: Option<fm_core::Currency>,
    due_date_input: utils::date_input::State,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
    add_transaction: Option<add_transaction::AddTransaction>,
    submitted: bool,
}

impl CreateBillView {
    pub fn new_with_transaction(transaction: fm_core::Transaction) -> Self {
        Self {
            transactions: vec![(transaction, fm_core::Sign::Negative)],
            ..Self::default()
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;

                let bill = locked_manager.get_bill(&id).await.unwrap().unwrap();

                let mut transactions = Vec::new();

                for (transaction_id, sign) in bill.transactions() {
                    transactions.push((
                        locked_manager
                            .get_transaction(*transaction_id)
                            .await
                            .unwrap()
                            .unwrap(),
                        *sign,
                    ));
                }
                Message::Initialize(bill, transactions)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::BillCreated(id) => {
                return Action::BillCreated(id);
            }
            Message::Initialize(bill, transactions) => {
                self.id = Some(*bill.id());
                bill.name().clone_into(&mut self.name_input);
                self.description_input = widget::text_editor::Content::with_text(
                    &bill.description().clone().unwrap_or_default(),
                );
                self.value = Some(bill.value().clone());
                self.due_date_input = utils::date_input::State::new(*bill.due_date());
                self.transactions = transactions;
                self.add_transaction = None;
            }
            Message::DueDateChanged(action) => {
                self.due_date_input.perform(action);
            }
            Message::NameInputChanged(name) => {
                self.name_input = name;
            }
            Message::ValueChanged(value) => {
                self.value = value;
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
                let due_date = self.due_date_input.date();
                let value = self.value.clone().unwrap();
                let mut transactions =
                    std::collections::HashMap::with_capacity(self.transactions.len());
                for transaction in &self.transactions {
                    transactions.insert(*transaction.0.id(), transaction.1);
                }
                if let Some(id) = id_option {
                    return Action::Task(iced::Task::future(async move {
                        finance_manager
                            .lock()
                            .await
                            .update_bill(id, name, description, value, transactions, due_date)
                            .unwrap()
                            .await
                            .unwrap();
                        Message::BillCreated(id)
                    }));
                } else {
                    return Action::Task(iced::Task::future(async move {
                        let mut locked_manager = finance_manager.lock().await;
                        let bill = locked_manager
                            .create_bill(name, description, value, transactions, due_date)
                            .unwrap()
                            .await
                            .unwrap();
                        drop(locked_manager);
                        Message::BillCreated(*bill.id())
                    }));
                }
            }
            Message::AddTransactionToggle => {
                if self.add_transaction.is_some() {
                    self.add_transaction = None;
                    return Action::None;
                }
                let ignored_transactions = self.transactions.iter().map(|x| *x.0.id()).collect();
                return Action::Task(iced::Task::future(async move {
                    Message::FetchedAddTransaction(
                        add_transaction::AddTransaction::fetch(
                            finance_manager,
                            ignored_transactions,
                        )
                        .await
                        .unwrap(),
                    )
                }));
            }
            Message::FetchedAddTransaction(add_transaction) => {
                self.add_transaction = Some(add_transaction);
            }
            Message::AddTransaction(transaction) => {
                self.transactions
                    .push((transaction, fm_core::Sign::Positive));
            }
            Message::ChangeTransactionSign(transaction_id, sign) => {
                self.transactions
                    .iter_mut()
                    .find(|(x, _)| x.id() == &transaction_id)
                    .unwrap()
                    .1 = sign;
            }
            Message::RemoveTransaction(transaction_id) => {
                self.transactions.retain(|x| *x.0.id() != transaction_id);
            }
            Message::AddTransactionMessage(m) => {
                if let Some(add_transaction) = &mut self.add_transaction {
                    match add_transaction.update(m, finance_manager) {
                        add_transaction::Action::Escape => {
                            self.add_transaction = None;
                        }
                        add_transaction::Action::AddTransaction(transaction) => {
                            self.transactions
                                .push((transaction, fm_core::Sign::Negative));
                        }
                        add_transaction::Action::Task(task) => {
                            return Action::Task(task.map(Message::AddTransactionMessage));
                        }
                        add_transaction::Action::None => {}
                    }
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        if let Some(add_transaction) = &self.add_transaction {
            return add_transaction.view().map(Message::AddTransactionMessage);
        }

        widget::column![
            utils::heading("Create Bill", utils::HeadingLevel::H1),
            utils::labeled_entry("Name", &self.name_input, Message::NameInputChanged, true),
            widget::row![
                "Description: ",
                widget::text_editor(&self.description_input)
                    .on_action(Message::DescriptionInputChanged)
            ]
            .spacing(10),
            widget::row![
                "Value: ",
                utils::CurrencyInput::new(self.value.clone(), Message::ValueChanged).required(true)
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            widget::row![
                "Due Date: ",
                utils::date_input::date_input(&self.due_date_input, "", false)
                    .view()
                    .map(Message::DueDateChanged),
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            "Transactions:",
            widget::container(
                utils::TableView::new(self.transactions.clone(), (), |(transaction, sign), _| {
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
                        widget::button("Delete")
                            .on_press(Message::RemoveTransaction(transaction_id))
                            .into(),
                        widget::text(transaction.title().clone()).into(),
                        widget::text(transaction.amount().to_num_string()).into(),
                        widget::text(
                            transaction
                                .date()
                                .to_offset(fm_core::get_local_timezone().unwrap())
                                .format(
                                    &time::format_description::parse("[day].[month].[year]")
                                        .unwrap(),
                                )
                                .unwrap(),
                        )
                        .into(),
                    ]
                })
                .headers(["", "", "Title", "Amount", "Date",])
                .sort_by(|a, b, column| match column {
                    0 => {
                        match (a.1, b.1) {
                            (fm_core::Sign::Positive, fm_core::Sign::Negative) => {
                                std::cmp::Ordering::Less
                            }
                            (fm_core::Sign::Negative, fm_core::Sign::Positive) => {
                                std::cmp::Ordering::Greater
                            }
                            _ => std::cmp::Ordering::Equal,
                        }
                    }
                    2 => a.0.title().cmp(b.0.title()),
                    3 => a.0.amount().cmp(&b.0.amount()),
                    4 => a.0.date().cmp(b.0.date()),
                    _ => panic!(),
                })
                .columns_sortable([true, false, true, true, true])
                .alignment(|_, _, _| (
                    iced::alignment::Horizontal::Left,
                    iced::alignment::Vertical::Center
                ))
            )
            .height(iced::Fill),
            widget::button("Add Transaction").on_press(Message::AddTransactionToggle),
            widget::button("Submit").on_press_maybe(if self.submittable() {
                Some(Message::Submit)
            } else {
                None
            }),
        ]
        .height(iced::Fill)
        .spacing(10)
        .into()
    }

    fn submittable(&self) -> bool {
        !self.name_input.is_empty() && self.value.is_some()
    }
}

mod add_transaction {
    use async_std::sync::Mutex;
    use std::sync::Arc;

    use anyhow::Result;

    use iced::widget;

    pub enum Action {
        Escape,
        AddTransaction(fm_core::Transaction),
        Task(iced::Task<Message>),
        None,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Back,
        FilterComponent(utils::filter_component::InnerMessage),
        AddTransaction(fm_core::Transaction),
        FetchedTransactions(Vec<fm_core::Transaction>),
    }

    #[derive(Debug, Clone)]
    pub struct AddTransaction {
        accounts: Vec<fm_core::account::Account>,
        categories: Vec<fm_core::Category>,
        bills: Vec<fm_core::Bill>,
        budgets: Vec<fm_core::Budget>,
        filter: Option<utils::filter_component::FilterComponent>,
        transactions: Vec<fm_core::Transaction>,
        ignored_transactions: Vec<fm_core::Id>,
    }

    impl AddTransaction {
        pub async fn fetch(
            finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
            ignored_transactions: Vec<fm_core::Id>,
        ) -> Result<Self> {
            let locked_manager = finance_manager.lock().await;
            let accounts = locked_manager.get_accounts().await?;
            let categories = locked_manager.get_categories().await?;
            let bills = locked_manager.get_bills().await?;
            let budgets = locked_manager.get_budgets().await?;
            Ok(Self {
                filter: Some(utils::filter_component::FilterComponent::new(
                    accounts.clone(),
                    categories.clone(),
                    bills.clone(),
                    budgets.clone(),
                )),
                accounts,
                categories,
                bills,
                budgets,
                transactions: Vec::new(),
                ignored_transactions,
            })
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
                                return Action::Task(iced::Task::future(async move {
                                    let locked_manager = finance_manager.lock().await;
                                    Message::FetchedTransactions(
                                        locked_manager
                                            .get_filtered_transactions(submitted_filter.clone())
                                            .await
                                            .unwrap(),
                                    )
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
                    Action::AddTransaction(transaction)
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
                    Action::None
                }
            }
        }

        pub fn view(&self) -> iced::Element<Message> {
            widget::column![
                utils::heading("Add", utils::HeadingLevel::H1),
                widget::button("Back").on_press(Message::Back),
                if let Some(filter) = &self.filter {
                    iced::Element::new(
                        widget::column![
                            "Create Filter for Transactions:",
                            filter.view().map(Message::FilterComponent),
                        ]
                        .spacing(10)
                        .width(iced::Length::Fill),
                    )
                } else {
                    utils::TableView::new(self.transactions.clone(), (), |x, _| {
                        [
                            widget::button("Add")
                                .on_press(Message::AddTransaction(x.clone()))
                                .into(),
                            widget::text(x.title().clone()).into(),
                            widget::text(x.amount().to_num_string()).into(),
                            widget::text(
                                x.date()
                                    .to_offset(fm_core::get_local_timezone().unwrap())
                                    .format(
                                        &time::format_description::parse("[day].[month].[year]")
                                            .unwrap(),
                                    )
                                    .unwrap(),
                            )
                            .into(),
                        ]
                    })
                    .headers([
                        "".to_string(),
                        "Title".to_string(),
                        "Amount".to_string(),
                        "Date".to_string(),
                    ])
                    .sort_by(|a, b, column| match column {
                        1 => a.title().cmp(b.title()),
                        2 => a.amount().cmp(&b.amount()),
                        3 => a.date().cmp(b.date()),
                        _ => panic!(),
                    })
                    .columns_sortable([false, true, true, true])
                    .into()
                }
            ]
            .spacing(10)
            .height(iced::Fill)
            .width(iced::Fill)
            .into()
        }
    }
}
