use super::super::{utils, AppMessage, View};

use anyhow::Result;

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
        async { BillOverview::fetch(finance_manager).await.unwrap() },
        |view| AppMessage::SwitchView(View::BillOverview(view)),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewBill(fm_core::Id),
    NewBill,
}

#[derive(Debug, Clone)]
pub struct BillOverview {
    bills: Vec<(fm_core::Bill, fm_core::Currency)>,
}

impl BillOverview {
    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let raw_bills = locked_manager.get_bills().await?;

        let mut bills = Vec::new();
        for bill in raw_bills {
            let sum = locked_manager.get_bill_sum(&bill).await?;
            bills.push((bill, sum));
        }

        Ok(Self { bills })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::ViewBill(bill_id) => (
                Some(View::Empty),
                super::bill::switch_view_command(bill_id, finance_manager),
            ),
            Message::NewBill => (
                Some(View::CreateBill(
                    super::create_bill::CreateBillView::default(),
                )),
                iced::Task::none(),
            ),
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
