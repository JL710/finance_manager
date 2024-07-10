use super::super::{AppMessage, View};

use anyhow::Result;

use async_std::sync::Mutex;
use std::sync::Arc;

use super::super::table_view::TableView;
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

#[derive(Debug, Clone)]
pub enum Message {
    DueDateChanged(Option<fm_core::DateTime>),
    NameInputChanged(String),
    ValueChanged(fm_core::Currency),
    DescriptionInputChanged(String),
    AddTransactionToggle,
    AddTransactionFilterSubmit(fm_core::transaction_filter::TransactionFilter),
    AddTransactionReloadTransactions(Vec<fm_core::Transaction>),
    FetchedAddTransaction(AddTransaction),
    AddTransaction(fm_core::Transaction),
    ChangeTransactionSign(fm_core::Id, fm_core::Sign),
    RemoveTransaction(fm_core::Id),
    Submit,
}

#[derive(Debug, Clone)]
pub struct CreateBillView {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: String,
    value: Option<fm_core::Currency>,
    due_date: Option<fm_core::DateTime>,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
    add_transaction: Option<AddTransaction>,
}

impl Default for CreateBillView {
    fn default() -> Self {
        Self {
            id: None,
            name_input: String::new(),
            description_input: String::new(),
            value: None,
            due_date: None,
            transactions: Vec::new(),
            add_transaction: None,
        }
    }
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
                    .get_transaction(transaction_id)
                    .await?
                    .unwrap(),
                sign,
            ));
        }

        Ok(Self {
            id: Some(*bill.id()),
            name_input: bill.name().to_owned(),
            description_input: bill.description().clone().unwrap_or(String::new()),
            value: Some(bill.value().clone()),
            due_date: bill.due_date().clone().map(|x| *x),
            transactions: transactions,
            add_transaction: None,
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::DueDateChanged(date) => {
                self.due_date = date;
            }
            Message::NameInputChanged(name) => {
                self.name_input = name;
            }
            Message::ValueChanged(value) => {
                self.value = Some(value);
            }
            Message::DescriptionInputChanged(description) => {
                self.description_input = description;
            }
            Message::Submit => {
                if !self.submittable() {
                    panic!("Cant Submit!")
                }
                let id_option = self.id.clone();
                let name = self.name_input.clone();
                let description = self.description_input.clone();
                let due_date = self.due_date.clone();
                let value = self.value.clone().unwrap();
                let transactions = self
                    .transactions
                    .clone()
                    .into_iter()
                    .map(|(transaction, sign)| (*transaction.id(), sign))
                    .collect::<Vec<_>>();
                if let Some(id) = id_option {
                    return (
                        Some(View::Empty),
                        iced::Task::perform(
                            async move {
                                let mut locked_manager = finance_manager.lock().await;
                                let bill = locked_manager
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
                                    .await
                                    .unwrap();
                                bill
                            },
                            |_| AppMessage::SwitchView(View::Empty),
                        ),
                    );
                } else {
                    return (
                        Some(View::Empty),
                        iced::Task::perform(
                            async move {
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
                                    .await
                                    .unwrap();
                                bill
                            },
                            |_| AppMessage::SwitchView(View::Empty),
                        ),
                    );
                }
            }
            Message::AddTransactionToggle => {
                if self.add_transaction.is_some() {
                    self.add_transaction = None;
                    return (None, iced::Task::none());
                }
                return (
                    None,
                    iced::Task::perform(
                        async { AddTransaction::fetch(finance_manager).await.unwrap() },
                        |x| AppMessage::CreateBillMessage(Message::FetchedAddTransaction(x)),
                    ),
                );
            }
            Message::FetchedAddTransaction(add_transaction) => {
                self.add_transaction = Some(add_transaction);
            }
            Message::AddTransactionFilterSubmit(filter) => {
                if let Some(add_transaction) = &mut self.add_transaction {
                    add_transaction.filter = None;
                }
                return (
                    None,
                    iced::Task::perform(
                        async move {
                            let locked_manager = finance_manager.lock().await;
                            let transactions = locked_manager
                                .get_filtered_transactions(filter.clone())
                                .await
                                .unwrap();
                            transactions
                        },
                        |x| {
                            AppMessage::CreateBillMessage(
                                Message::AddTransactionReloadTransactions(x),
                            )
                        },
                    ),
                );
            }
            Message::AddTransactionReloadTransactions(transactions) => {
                if let Some(add_transaction) = &mut self.add_transaction {
                    let mut transactions = transactions;
                    transactions.retain(|x| {
                        for (t, _) in &self.transactions {
                            if t.id() == x.id() {
                                return false;
                            }
                        }
                        return true;
                    });

                    add_transaction.transactions = transactions;
                }
            }
            Message::AddTransaction(transaction) => {
                self.transactions
                    .push((transaction, fm_core::Sign::Positive));
                self.add_transaction = None;
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
        }
        (None, iced::Task::none())
    }

    pub fn view(&self) -> iced::Element<Message> {
        if let Some(add_transaction) = &self.add_transaction {
            return add_transaction.view();
        }

        widget::column![
            widget::row![
                "Name: ",
                widget::text_input("Name", &self.name_input).on_input(Message::NameInputChanged),
            ]
            .spacing(10),
            widget::row![
                "Description: ",
                widget::text_input("Description", &self.description_input)
                    .on_input(Message::DescriptionInputChanged),
            ]
            .spacing(10),
            widget::row![
                "Value: ",
                super::super::currency_input::CurrencyInput::new(Message::ValueChanged)
                    .into_element(),
                iced::widget::horizontal_space().width(iced::Length::Fixed(5.0))
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            widget::row![
                "Due Date: ",
                super::super::date_input::DateInput::new(Message::DueDateChanged).into_element(),
            ]
            .width(iced::Length::Fill)
            .spacing(10),
            "Transactions:",
            TableView::new(self.transactions.clone(), |(transaction, sign)| {
                let transaction_id = transaction.id().clone();
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

#[derive(Debug, Clone)]
struct AddTransaction {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    filter: Option<fm_core::transaction_filter::TransactionFilter>,
    transactions: Vec<fm_core::Transaction>,
}

impl AddTransaction {
    async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let accounts = locked_manager.get_accounts().await?;
        let categories = locked_manager.get_categories().await?;
        Ok(Self {
            accounts,
            categories,
            filter: Some(fm_core::transaction_filter::TransactionFilter::default()),
            transactions: Vec::new(),
        })
    }

    fn view(&self) -> iced::Element<Message> {
        widget::column![
            widget::button("Back").on_press(Message::AddTransactionToggle),
            if let Some(filter) = &self.filter {
                super::super::filter_component::FilterComponent::new(
                    filter.clone(),
                    Message::AddTransactionFilterSubmit,
                    &self.accounts,
                    &self.categories,
                )
                .into_element()
            } else {
                TableView::new(self.transactions.clone(), |x| {
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
                .into_element()
            }
        ]
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
