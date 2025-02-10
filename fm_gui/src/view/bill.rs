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
    accounts: Vec<fm_core::account::Account>,
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
pub enum View {
    NotLoaded,
    Loaded {
        bill: fm_core::Bill,
        bill_sum: fm_core::Currency,
        transaction_table: utils::table_view::State<
            (fm_core::Transaction, fm_core::Sign),
            Vec<fm_core::account::Account>,
        >,
    },
}

impl View {
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
                let accounts = locked_manager.get_accounts().await.unwrap();
                Message::Initialize(Box::new(Init {
                    bill,
                    bill_sum,
                    transactions,
                    accounts,
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
                    transaction_table: utils::table_view::State::new(
                        init.transactions,
                        init.accounts,
                    )
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
                        5 => a.0.source().cmp(b.0.source()),
                        6 => a.0.destination().cmp(b.0.destination()),
                        _ => std::cmp::Ordering::Equal,
                    })
                    .sortable_columns([0, 1, 2, 3, 4, 5, 6]),
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
                        utils::table_view::Action::None => {}
                        utils::table_view::Action::OuterMessage(m) => {
                            return self.update(MessageContainer(m), finance_manager);
                        }
                        utils::table_view::Action::Task(task) => {
                            return Action::Task(
                                task.map(Message::TransactionTable).map(MessageContainer),
                            )
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
            super::view(
                "Bill",
                utils::spaced_column![
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
                                bill.due_date()
                                    .map_or(String::new(), utils::convert_date_time_to_date_string)
                            ),
                            utils::spal_row!["Sum: ", utils::colored_currency_display(bill_sum),]
                        ],
                        widget::horizontal_space(),
                        utils::spaced_column![
                            utils::button::edit(Some(Message::Edit)),
                            utils::button::delete(Some(Message::Delete))
                        ]
                    ],
                    widget::horizontal_rule(10),
                    utils::table_view::table_view(transaction_table)
                        .headers([
                            "Negative",
                            "Title",
                            "Description",
                            "Amount",
                            "Date",
                            "Source",
                            "Destination"
                        ])
                        .view(|(transaction, sign), accounts| [
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
                            widget::text(utils::convert_date_time_to_date_string(
                                *transaction.date()
                            ))
                            .into(),
                            widget::text(
                                accounts
                                    .iter()
                                    .find(|acc| acc.id() == transaction.source())
                                    .unwrap()
                                    .name(),
                            )
                            .into(),
                            widget::text(
                                accounts
                                    .iter()
                                    .find(|acc| acc.id() == transaction.destination())
                                    .unwrap()
                                    .name(),
                            )
                            .into(),
                        ])
                        .map(Message::TransactionTable),
                ]
                .height(iced::Fill),
            )
            .map(MessageContainer)
        } else {
            super::view("Bill", "Loading...")
        }
    }
}
