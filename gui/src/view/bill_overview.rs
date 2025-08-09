use anyhow::Result;
use iced::widget;

pub enum Action {
    ViewBill(fm_core::Id),
    NewBill,
    None,
    Task(iced::Task<MessageContainer>),
}

#[derive(Debug, Clone)]
struct Init {
    bills: Vec<(fm_core::Bill, fm_core::Currency)>,
    closed: bool,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    ViewBill(fm_core::Id),
    BillTable(components::table_view::InnerMessage<Message>),
    NewBill,
    Initialize(Init),
    Reload(Init),
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

    pub fn reload(
        &mut self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> iced::Task<MessageContainer> {
        error::failing_task(init_future(finance_controller, false))
            .map(Message::Reload)
            .map(MessageContainer)
    }

    pub fn fetch_unclosed(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::new(Vec::new(), false),
            error::failing_task(init_future(finance_controller, false))
                .map(Message::Initialize)
                .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        let message = message.0;
        match message {
            Message::Reload(init) => {
                self.closed = init.closed;
                self.bill_table.edit_items(|items| *items = init.bills);
                Action::None
            }
            Message::Initialize(init) => {
                self.bill_table.set_items(init.bills);
                self.closed = init.closed;
                Action::None
            }
            Message::ViewClosed => Action::Task(
                error::failing_task(init_future(finance_controller, true))
                    .map(Message::Initialize)
                    .map(MessageContainer),
            ),
            Message::BackToUnclosed => Action::Task(
                error::failing_task(init_future(finance_controller, false))
                    .map(Message::Initialize)
                    .map(MessageContainer),
            ),
            Message::ViewBill(bill_id) => Action::ViewBill(bill_id),
            Message::NewBill => Action::NewBill,
            Message::BillTable(inner) => match self.bill_table.perform(inner) {
                components::table_view::Action::OuterMessage(m) => {
                    self.update(MessageContainer(m), finance_controller)
                }
                components::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::BillTable).map(MessageContainer))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        iced::Element::new(
            components::overlap_bottom_right(
                components::back_prev_container(
                    if self.closed {
                        "Closed Bills"
                    } else {
                        "Unclosed Bills"
                    },
                    if self.closed {
                        Some(("Back to Unclosed", Some(Message::BackToUnclosed)))
                    } else {
                        None
                    },
                    if !self.closed {
                        Some(("Closed Bills", Some(Message::ViewClosed)))
                    } else {
                        None
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
                                widget::text(bill.0.due_date.map_or(String::new(), |x| {
                                    components::date_time::to_date_string(x.date())
                                }))
                                .into(),
                                widget::text(bill.0.transactions.len()).into(),
                            ]
                        })
                        .map(Message::BillTable),
                ),
                components::button::large_round_plus_button(Some(Message::NewBill)),
            )
            .height(iced::Fill)
            .width(iced::Fill),
        )
        .map(MessageContainer)
    }
}

async fn init_future(
    finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    closed: bool,
) -> Result<Init> {
    let bills = finance_controller.get_bills(Some(closed)).await?;
    let mut bill_tuples = Vec::new();
    for bill in bills {
        let sum = finance_controller.get_bill_sum(&bill).await?;
        bill_tuples.push((bill, sum));
    }
    Ok(Init {
        bills: bill_tuples,
        closed,
    })
}
