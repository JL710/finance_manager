use super::super::{utils, AppMessage, View};

use anyhow::Result;

use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
        async move { CreateBillView::fetch(id, finance_manager).await.unwrap() },
        |view| AppMessage::SwitchView(View::CreateBill(view)),
    )
}

pub enum Action {
    None,
    CreateBill(iced::Task<fm_core::Id>),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    DueDateChanged(Option<fm_core::DateTime>),
    NameInputChanged(String),
    ValueChanged(Option<fm_core::Currency>),
    DescriptionInputChanged(utils::multiline_text_input::Action),
    AddTransactionToggle,
    AddTransaction(fm_core::Transaction),
    ChangeTransactionSign(fm_core::Id, fm_core::Sign),
    RemoveTransaction(fm_core::Id),
    AddTransactionMessage(add_transaction::Message),
    FetchedAddTransaction(add_transaction::AddTransaction),
    Submit,
    Initialize(fm_core::Bill, Vec<(fm_core::Transaction, fm_core::Sign)>),
}

#[derive(Debug, Clone, Default)]
pub struct CreateBillView {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: utils::multiline_text_input::State,
    value: Option<fm_core::Currency>,
    due_date: Option<fm_core::DateTime>,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
    add_transaction: Option<add_transaction::AddTransaction>,
}

impl CreateBillView {
    pub async fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;

        let bill = locked_manager.get_bill(&id).await?.unwrap();

        let mut transactions = Vec::new();

        for (transaction_id, sign) in bill.transactions() {
            transactions.push((
                locked_manager
                    .get_transaction(*transaction_id)
                    .await?
                    .unwrap(),
                *sign,
            ));
        }

        Ok(Self {
            id: Some(*bill.id()),
            name_input: bill.name().to_owned(),
            description_input: utils::multiline_text_input::State::new(
                bill.description().clone().unwrap_or(String::new()),
            ),
            value: Some(bill.value().clone()),
            due_date: *bill.due_date(),
            transactions,
            add_transaction: None,
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::Initialize(bill, transactions) => {
                self.id = Some(*bill.id());
                self.name_input = bill.name().to_owned();
                self.description_input = utils::multiline_text_input::State::new(
                    bill.description().clone().unwrap_or(String::new()),
                );
                self.value = Some(bill.value().clone());
                self.due_date = *bill.due_date();
                self.transactions = transactions;
                self.add_transaction = None;
            }
            Message::DueDateChanged(date) => {
                self.due_date = date;
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
                let id_option = self.id;
                let name = self.name_input.clone();
                let description = self.description_input.get_content().clone();
                let due_date = self.due_date;
                let value = self.value.clone().unwrap();
                let transactions = self
                    .transactions
                    .clone()
                    .into_iter()
                    .map(|(transaction, sign)| (*transaction.id(), sign))
                    .collect::<Vec<_>>();
                if let Some(id) = id_option {
                    let manager = finance_manager.clone();
                    return Action::CreateBill(iced::Task::future(async move {
                        finance_manager
                            .lock()
                            .await
                            .update_bill(
                                id,
                                name,
                                if description.is_empty() {
                                    None
                                } else {
                                    Some(description)
                                },
                                value,
                                transactions,
                                due_date,
                            )
                            .unwrap()
                            .await
                            .unwrap();
                        id
                    }));
                } else {
                    let manager = finance_manager.clone();
                    return Action::CreateBill(iced::Task::future(async move {
                        let mut locked_manager = finance_manager.lock().await;
                        let bill = locked_manager
                            .create_bill(
                                name,
                                if description.is_empty() {
                                    None
                                } else {
                                    Some(description)
                                },
                                value,
                                transactions,
                                due_date,
                            )
                            .unwrap()
                            .await
                            .unwrap();
                        drop(locked_manager);
                        *bill.id()
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
                                .push((transaction, fm_core::Sign::Positive));
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
            widget::row![
                "Name: ",
                widget::text_input("Name", &self.name_input).on_input(Message::NameInputChanged),
            ]
            .spacing(10),
            widget::row![
                "Description: ",
                utils::multiline_text_input::MultilineTextInput::new(
                    self.description_input.clone(),
                )
                .on_action(Message::DescriptionInputChanged)
                .view()
            ]
            .spacing(10),
            widget::row![
                "Value: ",
                utils::CurrencyInput::new(Message::ValueChanged).into_element(),
                iced::widget::horizontal_space().width(iced::Length::Fixed(5.0))
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            widget::row![
                "Due Date: ",
                utils::DateInput::new(Message::DueDateChanged).into_element(),
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            "Transactions:",
            utils::TableView::new(self.transactions.clone(), |(transaction, sign)| {
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
                    widget::text(transaction.date().format("%d.%m.%Y").to_string()).into(),
                ]
            })
            .headers(["", "", "Title", "Amount", "Date",])
            .alignment(|_, _, _| (
                iced::alignment::Horizontal::Left,
                iced::alignment::Vertical::Center
            ))
            .into_element(),
            widget::button("Add Transaction").on_press(Message::AddTransactionToggle),
            widget::button("Submit").on_press_maybe(if self.submittable() {
                Some(Message::Submit)
            } else {
                None
            }),
        ]
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

    use crate::utils;

    pub enum Action {
        Escape,
        AddTransaction(fm_core::Transaction),
        Task(iced::Task<Message>),
        None,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        Back,
        FilterSubmit(fm_core::transaction_filter::TransactionFilter),
        AddTransaction(fm_core::Transaction),
        FetchedTransactions(Vec<fm_core::Transaction>),
    }

    #[derive(Debug, Clone)]
    pub struct AddTransaction {
        accounts: Vec<fm_core::account::Account>,
        categories: Vec<fm_core::Category>,
        filter: Option<fm_core::transaction_filter::TransactionFilter>,
        transactions: Vec<fm_core::Transaction>,
        ignored_transactions: Vec<fm_core::Id>,
    }

    impl AddTransaction {
        pub async fn fetch(
            finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
            ignored_transactions: Vec<fm_core::Id>,
        ) -> Result<Self> {
            let locked_manager = finance_manager.lock().await;
            let accounts = locked_manager.get_accounts().await?;
            let categories = locked_manager.get_categories().await?;
            Ok(Self {
                accounts,
                categories,
                filter: Some(fm_core::transaction_filter::TransactionFilter::default()),
                transactions: Vec::new(),
                ignored_transactions,
            })
        }

        pub fn update(
            &mut self,
            message: Message,
            finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        ) -> Action {
            match message {
                Message::Back => Action::Escape,
                Message::FilterSubmit(filter) => {
                    self.filter = None;
                    Action::Task(iced::Task::future(async move {
                        let locked_manager = finance_manager.lock().await;
                        Message::FetchedTransactions(
                            locked_manager
                                .get_filtered_transactions(filter.clone())
                                .await
                                .unwrap(),
                        )
                    }))
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
                    widget::column![
                        "Create Filter for Transactions:",
                        utils::FilterComponent::new(
                            filter.clone(),
                            Message::FilterSubmit,
                            &self.accounts,
                            &self.categories,
                        )
                        .into_element()
                    ]
                    .spacing(10)
                    .width(iced::Length::Fill)
                    .into()
                } else {
                    utils::TableView::new(self.transactions.clone(), |x| {
                        [
                            widget::button("Add")
                                .on_press(Message::AddTransaction(x.clone()))
                                .into(),
                            widget::text(x.title().clone()).into(),
                            widget::text(x.amount().to_num_string()).into(),
                            widget::text(x.date().format("%d.%m.%Y").to_string()).into(),
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
                    .into_element()
                }
            ]
            .spacing(10)
            .width(iced::Length::Fill)
            .into()
        }
    }
}
