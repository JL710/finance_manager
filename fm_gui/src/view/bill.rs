use super::super::{AppMessage, View};

use async_std::sync::Mutex;
use std::sync::Arc;

use super::super::table_view::TableView;
use super::super::utils;
use iced::widget;

use anyhow::Result;

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::future(async move {
        let view = Bill::fetch(&id, finance_manager).await.unwrap();
        AppMessage::SwitchView(View::ViewBill(view))
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewTransaction(fm_core::Id),
    Edit,
}

#[derive(Debug, Clone)]
pub struct Bill {
    bill: fm_core::Bill,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
}

impl Bill {
    pub async fn fetch(
        id: &fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let bill = locked_manager.get_bill(id).await?.unwrap();

        let mut transactions = Vec::new();
        for (transaction_id, sign) in bill.transactions() {
            let transaction = locked_manager
                .get_transaction(*transaction_id)
                .await?
                .unwrap();
            transactions.push((transaction, *sign));
        }

        Ok(Self { bill, transactions })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::ViewTransaction(transaction_id) => (
                Some(View::Empty),
                super::transaction::switch_view_command(transaction_id, finance_manager),
            ),
            Message::Edit => (
                Some(View::Empty),
                super::create_bill::switch_view_command(*self.bill.id(), finance_manager),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
            widget::row![
                widget::column![
                    widget::text!("Name: {}", self.bill.name()),
                    widget::text!(
                        "Description: {}",
                        self.bill.description().clone().unwrap_or_default()
                    ),
                    widget::text!("Amount: {}€", self.bill.value().to_num_string()),
                    widget::text!(
                        "Due Date: {}",
                        self.bill
                            .due_date()
                            .map_or(String::new(), |d| d.format("%d.%m.%Y").to_string())
                    ),
                ],
                widget::horizontal_space(),
                widget::button("Edit").on_press(Message::Edit),
            ],
            widget::scrollable(
                TableView::new(self.transactions.clone(), |(transaction, sign)| [
                    widget::checkbox("Negative", *sign == fm_core::Sign::Negative).into(),
                    widget::button(widget::text(transaction.title().clone()))
                        .style(utils::button_link_style)
                        .padding(0)
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
    }
}
