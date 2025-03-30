use anyhow::Result;
use iced::widget;

pub enum Action {
    ViewBill(fm_core::Id),
    NewBill,
    None,
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewBill(fm_core::Id),
    BillTable(components::table_view::InnerMessage<Message>),
    NewBill,
    Initialize(Vec<(fm_core::Bill, fm_core::Currency)>, bool),
    ViewClosed,
    BackToUnclosed,
}

#[derive(Debug)]
pub struct View {
    bill_table: components::table_view::State<(fm_core::Bill, fm_core::Currency), ()>,
    closed: bool,
}

impl View {
    pub fn new(bills: Vec<(fm_core::Bill, fm_core::Currency)>, closed: bool) -> Self {
        Self {
            bill_table: components::table_view::State::new(bills, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name.cmp(&b.0.name),
                    1 => a.0.value.cmp(&b.0.value),
                    2 => a.1.cmp(&b.1),
                    3 => a.0.due_date.cmp(&b.0.due_date),
                    4 => a.0.transactions.len().cmp(&b.0.transactions.len()),
                    _ => {
                        panic!()
                    }
                })
                .sortable_columns([0, 1, 2, 3, 4]),
            closed,
        }
    }

    pub fn fetch_unclosed(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new(), false),
            error::failing_task(init_future(finance_controller, false)),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Initialize(bills, closed) => {
                self.bill_table.set_items(bills);
                self.closed = closed;
                Action::None
            }
            Message::ViewClosed => {
                Action::Task(error::failing_task(init_future(finance_controller, true)))
            }
            Message::BackToUnclosed => {
                Action::Task(error::failing_task(init_future(finance_controller, false)))
            }
            Message::ViewBill(bill_id) => Action::ViewBill(bill_id),
            Message::NewBill => Action::NewBill,
            Message::BillTable(inner) => match self.bill_table.perform(inner) {
                components::table_view::Action::OuterMessage(m) => {
                    self.update(m, finance_controller)
                }
                components::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::BillTable))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        components::overlap_bottom_right(
            components::spaced_column![
                if self.closed {
                    widget::row![
                        widget::button(components::spal_row![
                            icons::arrow_left(),
                            "Back to Unclosed"
                        ])
                        .on_press(Message::BackToUnclosed)
                    ]
                } else {
                    widget::row![
                        widget::horizontal_space(),
                        widget::button(components::spal_row!["Closed Bills", icons::arrow_right()])
                            .on_press(Message::ViewClosed)
                    ]
                },
                if self.closed {
                    "Closed Bills:"
                } else {
                    "Unclosed Bills:"
                },
                components::table_view::table_view(&self.bill_table)
                    .headers(["Name", "Value", "Sum", "Due Date", "Transaction"])
                    .view(|bill, _| {
                        [
                            components::link(bill.0.name.as_str())
                                .on_press(Message::ViewBill(bill.0.id))
                                .into(),
                            widget::text!("{}€", bill.0.value.to_num_string()).into(),
                            components::colored_currency_display(&bill.1),
                            widget::text(
                                bill.0
                                    .due_date
                                    .map_or(String::new(), components::date_time::to_date_string),
                            )
                            .into(),
                            widget::text(bill.0.transactions.len()).into(),
                        ]
                    })
                    .map(Message::BillTable),
            ],
            components::button::large_round_plus_button(Some(Message::NewBill)),
        )
        .height(iced::Fill)
        .width(iced::Fill)
        .into()
    }
}

async fn init_future(
    finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    closed: bool,
) -> Result<Message> {
    let bills = finance_controller.get_bills(Some(closed)).await?;
    let mut bill_tuples = Vec::new();
    for bill in bills {
        let sum = finance_controller.get_bill_sum(&bill).await?;
        bill_tuples.push((bill, sum));
    }
    Ok(Message::Initialize(bill_tuples, closed))
}
