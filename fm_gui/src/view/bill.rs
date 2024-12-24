use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    Edit(fm_core::Id),
    Task(iced::Task<MessageContainer>),
    Deleted,
}

#[derive(Debug, Clone)]
struct Init {
    bill: fm_core::Bill,
    bill_sum: fm_core::Currency,
    transactions: Vec<(fm_core::Transaction, fm_core::Sign)>,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    ViewTransaction(fm_core::Id),
    Edit,
    Initialize(Box<Init>),
    Delete,
    Deleted,
    TransactionTable(utils::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Bill {
    NotLoaded,
    Loaded {
        bill: fm_core::Bill,
        bill_sum: fm_core::Currency,
        transaction_table: utils::table_view::State<(fm_core::Transaction, fm_core::Sign), (), 5>,
    },
}

impl Bill {
    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<MessageContainer>) {
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
                Message::Initialize(Box::new(Init {
                    bill,
                    bill_sum,
                    transactions,
                }))
            })
            .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message.0 {
            Message::ViewTransaction(transaction_id) => Action::ViewTransaction(transaction_id),
            Message::Edit => {
                if let Self::Loaded { bill, .. } = self {
                    Action::Edit(*bill.id())
                } else {
                    Action::None
                }
            }
            Message::Initialize(init) => {
                *self = Self::Loaded {
                    bill: init.bill,
                    bill_sum: init.bill_sum,
                    transaction_table: utils::table_view::State::new(init.transactions, ())
                        .sort_by(|a, b, column| match column {
                            0 => match (a.1, b.1) {
                                (fm_core::Sign::Positive, fm_core::Sign::Negative) => {
                                    std::cmp::Ordering::Greater
                                }
                                (fm_core::Sign::Negative, fm_core::Sign::Positive) => {
                                    std::cmp::Ordering::Less
                                }
                                _ => std::cmp::Ordering::Equal,
                            },
                            1 => a.0.title().cmp(b.0.title()),
                            2 => {
                                a.0.description()
                                    .unwrap_or("")
                                    .cmp(b.0.description().unwrap_or(""))
                            }
                            3 => a.0.amount().cmp(&b.0.amount()),
                            4 => a.0.date().cmp(b.0.date()),
                            _ => std::cmp::Ordering::Equal,
                        })
                        .sortable_columns([true, true, true, true, true]),
                };
                Action::None
            }
            Message::Delete => {
                if let Self::Loaded { bill, .. } = self {
                    if let rfd::MessageDialogResult::No = rfd::MessageDialog::new()
                        .set_title("Delete Bill")
                        .set_description("Are you sure you want to delete this bill?")
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        return Action::None;
                    }

                    let bill_id = *bill.id();
                    Action::Task(
                        iced::Task::future(async move {
                            finance_manager
                                .lock()
                                .await
                                .delete_bill(bill_id)
                                .await
                                .unwrap();
                            Message::Deleted
                        })
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
            }
            Message::Deleted => Action::Deleted,
            Message::TransactionTable(inner) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    match transaction_table.perform(inner) {
                        utils::table_view::Action::PageChange(_) => {}
                        utils::table_view::Action::None => {}
                        utils::table_view::Action::OuterMessage(m) => {
                            return self.update(MessageContainer(m), finance_manager);
                        }
                    }
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<MessageContainer> {
        if let Self::Loaded {
            bill,
            bill_sum,
            transaction_table,
        } = self
        {
            iced::Element::new(
                widget::column![
                    utils::heading("Bill", utils::HeadingLevel::H1),
                    widget::row![
                        widget::column![
                            widget::text!("Name: {}", bill.name()),
                            widget::row![
                                "Description: ",
                                widget::container(widget::text(
                                    bill.description().clone().unwrap_or_default()
                                ))
                                .style(utils::style::container_style_background_weak)
                            ],
                            widget::text!("Amount: {}€", bill.value().to_num_string()),
                            widget::text!(
                                "Due Date: {}",
                                bill.due_date().map_or(String::new(), |d| d
                                    .to_offset(fm_core::get_local_timezone().unwrap())
                                    .format(
                                        &time::format_description::parse("[day].[month].[year]")
                                            .unwrap()
                                    )
                                    .unwrap())
                            ),
                            widget::row!["Sum: ", utils::colored_currency_display(bill_sum),]
                                .spacing(10)
                        ],
                        widget::horizontal_space(),
                        widget::column![
                            widget::button("Edit").on_press(Message::Edit),
                            widget::button("Delete")
                                .on_press(Message::Delete)
                                .style(widget::button::danger),
                        ]
                        .spacing(10)
                    ],
                    widget::horizontal_rule(10),
                    utils::table_view::table_view(transaction_table)
                        .headers(["Negative", "Title", "Description", "Amount", "Date"])
                        .view(|(transaction, sign), _| [
                            widget::checkbox("Positive", *sign == fm_core::Sign::Positive).into(),
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
                            widget::text(
                                transaction
                                    .date()
                                    .to_offset(fm_core::get_local_timezone().unwrap())
                                    .format(
                                        &time::format_description::parse("[day].[month].[year]")
                                            .unwrap()
                                    )
                                    .unwrap()
                            )
                            .into(),
                        ])
                        .map(Message::TransactionTable),
                ]
                .height(iced::Fill)
                .spacing(10),
            )
            .map(MessageContainer)
        } else {
            widget::text("Loading...").into()
        }
    }
}
