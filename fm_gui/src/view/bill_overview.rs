use super::super::utils;

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    ViewBill(fm_core::Id),
    NewBill,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewBill(fm_core::Id),
    NewBill,
    Initialize(Vec<(fm_core::Bill, fm_core::Currency)>),
}

#[derive(Debug, Clone)]
pub struct BillOverview {
    bills: Vec<(fm_core::Bill, fm_core::Currency)>,
}

impl BillOverview {
    pub fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self { bills: Vec::new() },
            iced::Task::future(async move {
                let bills = finance_manager.lock().await.get_bills().await.unwrap();
                let mut bill_tuples = Vec::new();
                for bill in bills {
                    let sum = finance_manager
                        .lock()
                        .await
                        .get_bill_sum(&bill)
                        .await
                        .unwrap();
                    bill_tuples.push((bill, sum));
                }
                Message::Initialize(bill_tuples)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::Initialize(bills) => {
                self.bills = bills;
                Action::None
            }
            Message::ViewBill(bill_id) => Action::ViewBill(bill_id),
            Message::NewBill => Action::NewBill,
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
            utils::heading("Bill Overview", utils::HeadingLevel::H1),
            widget::button("New").on_press(Message::NewBill),
            utils::TableView::new(self.bills.clone(), |bill| [
                utils::link(widget::text(bill.0.name().clone()))
                    .on_press(Message::ViewBill(*bill.0.id()))
                    .into(),
                widget::text!("{}€", bill.0.value().to_num_string()).into(),
                utils::colored_currency_display(&bill.1),
                widget::text(
                    bill.0
                        .due_date()
                        .map_or(String::new(), |x| x.format("%d.%m.%Y").to_string())
                )
                .into(),
                widget::text(bill.0.transactions().len()).into()
            ])
            .headers(["Name", "Value", "Sum", "Due Date", "Transaction"])
            .sort_by(|a, b, column| {
                match column {
                    0 => a.0.name().cmp(b.0.name()),
                    1 => a.0.value().cmp(b.0.value()),
                    2 => a.0.due_date().cmp(b.0.due_date()),
                    3 => a.1.cmp(&b.1),
                    4 => a.0.transactions().len().cmp(&b.0.transactions().len()),
                    _ => {
                        panic!()
                    }
                }
            })
            .columns_sortable([true, true, true, true, true])
            .into_element()
        ]
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
