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
    BillTable(utils::table_view::InnerMessage<Message>),
    NewBill,
    Initialize(Vec<(fm_core::Bill, fm_core::Currency)>),
}

#[derive(Debug)]
pub struct BillOverview {
    bill_table: utils::table_view::State<(fm_core::Bill, fm_core::Currency), ()>,
}

impl BillOverview {
    pub fn new(bills: Vec<(fm_core::Bill, fm_core::Currency)>) -> Self {
        Self {
            bill_table: utils::table_view::State::new(bills, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name().cmp(b.0.name()),
                    1 => a.0.value().cmp(b.0.value()),
                    2 => a.1.cmp(&b.1),
                    3 => a.0.due_date().cmp(b.0.due_date()),
                    4 => a.0.transactions().len().cmp(&b.0.transactions().len()),
                    _ => {
                        panic!()
                    }
                })
                .sortable_columns([0, 1, 2, 3, 4]),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new()),
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
        _finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize(bills) => {
                self.bill_table.set_items(bills);
                Action::None
            }
            Message::ViewBill(bill_id) => Action::ViewBill(bill_id),
            Message::NewBill => Action::NewBill,
            Message::BillTable(inner) => match self.bill_table.perform(inner) {
                utils::table_view::Action::OuterMessage(m) => self.update(m, _finance_manager),
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Bill Overview",
            utils::spaced_column![
                widget::button("New").on_press(Message::NewBill),
                utils::table_view::table_view(&self.bill_table)
                    .headers(["Name", "Value", "Sum", "Due Date", "Transaction"])
                    .view(|bill, _| [
                        utils::link(widget::text(bill.0.name().clone()))
                            .on_press(Message::ViewBill(*bill.0.id()))
                            .into(),
                        widget::text!("{}€", bill.0.value().to_num_string()).into(),
                        utils::colored_currency_display(&bill.1),
                        widget::text(bill.0.due_date().map_or(String::new(), |x| {
                            x.to_offset(fm_core::get_local_timezone().unwrap())
                                .format(
                                    &time::format_description::parse("[day].[month].[year]")
                                        .unwrap(),
                                )
                                .unwrap()
                        }))
                        .into(),
                        widget::text(bill.0.transactions().len()).into()
                    ])
                    .map(Message::BillTable),
            ]
            .height(iced::Fill)
            .width(iced::Fill),
        )
    }
}
