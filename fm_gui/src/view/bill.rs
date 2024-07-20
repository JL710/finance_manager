use super::super::utils;

use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    Edit(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewTransaction(fm_core::Id),
    Edit,
    Initialize(
        fm_core::Bill,
        fm_core::Currency,
        Vec<(fm_core::Transaction, fm_core::Sign)>,
    ),
}

#[derive(Debug, Clone)]
pub enum Bill {
    NotLoaded,
    Loaded {
        bill: fm_core::Bill,
        bill_sum: fm_core::Currency,
        transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
    },
}

impl Bill {
    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let bill = locked_manager.get_bill(&id).await.unwrap().unwrap();

                let bill_sum = locked_manager.get_bill_sum(&bill).await.unwrap();

                let mut transactions = Vec::new();
                for (transaction_id, sign) in bill.transactions() {
                    let transaction = locked_manager
                        .get_transaction(*transaction_id)
                        .await
                        .unwrap()
                        .unwrap();
                    transactions.push((transaction, *sign));
                }
                Message::Initialize(bill, bill_sum, transactions)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::ViewTransaction(transaction_id) => Action::ViewTransaction(transaction_id),
            Message::Edit => {
                if let Self::Loaded { bill, .. } = self {
                    Action::Edit(*bill.id())
                } else {
                    Action::None
                }
            }
            Message::Initialize(bill, sum, transactions) => {
                *self = Self::Loaded {
                    bill,
                    bill_sum: sum,
                    transactions,
                };
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        if let Self::Loaded {
            bill,
            bill_sum,
            transactions,
        } = self
        {
            widget::column![
                utils::heading("Bill", utils::HeadingLevel::H1),
                widget::row![
                    widget::column![
                        widget::text!("Name: {}", bill.name()),
                        widget::row![
                            "Description: ",
                            widget::text(bill.description().clone().unwrap_or_default())
                        ],
                        widget::text!("Amount: {}€", bill.value().to_num_string()),
                        widget::text!(
                            "Due Date: {}",
                            bill.due_date()
                                .map_or(String::new(), |d| d.format("%d.%m.%Y").to_string())
                        ),
                        widget::row!["Sum: ", utils::colored_currency_display(bill_sum),]
                            .spacing(10)
                    ],
                    widget::horizontal_space(),
                    widget::button("Edit").on_press(Message::Edit),
                ],
                widget::horizontal_rule(10),
                widget::scrollable(
                    utils::TableView::new(transactions.clone(), |(transaction, sign)| [
                        widget::checkbox("Negative", *sign == fm_core::Sign::Negative).into(),
                        utils::link(widget::text(transaction.title().clone()))
                            .on_press(Message::ViewTransaction(*transaction.id()))
                            .into(),
                        widget::text(
                            transaction
                                .description()
                                .map_or(String::new(), |x| x.to_string())
                        )
                        .into(),
                        widget::text!("{}€", transaction.amount().to_num_string()).into(),
                        widget::text(transaction.date().format("%d.%m.%Y").to_string()).into(),
                    ])
                    .headers(["Negative", "Title", "Description", "Amount", "Date"])
                    .into_element()
                )
            ]
            .spacing(10)
            .into()
        } else {
            widget::text("Loading...").into()
        }
    }
}
