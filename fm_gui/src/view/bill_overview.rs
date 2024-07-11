use crate::utils;

use super::super::{AppMessage, View};

use anyhow::Result;

use async_std::sync::Mutex;
use std::sync::Arc;

use super::super::table_view::TableView;
use iced::widget;

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
    bills: Vec<fm_core::Bill>,
}

impl BillOverview {
    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let bills = locked_manager.get_bills().await?;

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
            widget::button("New").on_press(Message::NewBill),
            TableView::new(self.bills.clone(), |bill| [
                widget::button(widget::text(bill.name().clone()))
                    .on_press(Message::ViewBill(*bill.id()))
                    .style(utils::button_link_style)
                    .padding(0)
                    .into(),
                widget::text!("{}€", bill.value().to_num_string()).into(),
                widget::text(
                    bill.due_date()
                        .map_or(String::new(), |x| x.format("%d.%m.%Y").to_string())
                )
                .into(),
                widget::text(bill.transactions().len()).into()
            ])
            .headers(["Name", "Value", "Due Date", "Transaction"])
            .into_element()
        ]
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
