use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

use super::super::utils;

pub enum Action {
    None,
    Edit(fm_core::Id),
    Delete(iced::Task<()>),
    ViewAccount(fm_core::Id),
    ViewBudget(fm_core::Id),
    NewBillWithTransaction(fm_core::Transaction),
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit,
    Delete,
    ViewAccount(fm_core::Id),
    ViewBudget(fm_core::Id),
    Initialize {
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        categories: Vec<fm_core::Category>,
    },
    NewBill,
}

#[derive(Debug, Clone)]
pub enum Transaction {
    NotLoaded,
    Loaded {
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        categories: Vec<fm_core::Category>,
    },
}

impl Transaction {
    pub fn new(
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        categories: Vec<fm_core::Category>,
    ) -> Self {
        Self::Loaded {
            transaction,
            source,
            destination,
            budget,
            categories,
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let transaction = locked_manager.get_transaction(id).await.unwrap().unwrap();
                let source = locked_manager
                    .get_account(*transaction.source())
                    .await
                    .unwrap()
                    .unwrap();
                let destination = locked_manager
                    .get_account(*transaction.destination())
                    .await
                    .unwrap()
                    .unwrap();
                let budget = match transaction.budget() {
                    Some(budget_id) => locked_manager.get_budget(budget_id.0).await.unwrap(),
                    None => None,
                };
                let categories = locked_manager.get_categories().await.unwrap();
                Message::Initialize {
                    transaction,
                    source,
                    destination,
                    budget,
                    categories,
                }
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize {
                transaction,
                source,
                destination,
                budget,
                categories,
            } => {
                *self = Self::Loaded {
                    transaction,
                    source,
                    destination,
                    budget,
                    categories,
                };
                Action::None
            }
            Message::NewBill => {
                if let Self::Loaded { transaction, .. } = self {
                    Action::NewBillWithTransaction(transaction.clone())
                } else {
                    Action::None
                }
            }
            Message::Edit => {
                if let Self::Loaded { transaction, .. } = self {
                    Action::Edit(*transaction.id())
                } else {
                    Action::None
                }
            }
            Message::Delete => {
                match rfd::MessageDialog::new()
                    .set_title("Delete Transaction")
                    .set_description("Are you sure you want to delete this transaction?")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()
                {
                    rfd::MessageDialogResult::Yes => (),
                    _ => return Action::None,
                }
                if let Self::Loaded { transaction, .. } = self {
                    let id = *transaction.id();
                    Action::Delete(iced::Task::future(async move {
                        finance_manager
                            .lock()
                            .await
                            .delete_transaction(id)
                            .await
                            .unwrap();
                    }))
                } else {
                    Action::None
                }
            }
            Message::ViewAccount(acc) => Action::ViewAccount(acc),
            Message::ViewBudget(budget) => Action::ViewBudget(budget),
        }
    }

    pub fn view(&self) -> iced::Element<'static, Message> {
        if let Self::Loaded {
            transaction,
            source,
            destination,
            budget,
            categories,
        } = self
        {
            let mut column = widget::column![
                widget::row![widget::text!("Value: {}", transaction.amount())],
                widget::text!("Name: {}", transaction.title()),
                utils::link(widget::text!("Source: {}", source))
                    .on_press(Message::ViewAccount(*source.id())),
                utils::link(widget::text!("Destination: {}", destination))
                    .on_press(Message::ViewAccount(*destination.id())),
                widget::text!("Date: {}", transaction.date().format("%d.%m.%Y")),
            ]
            .spacing(10);

            if let Some(budget) = &budget {
                column = column.push(
                    widget::row![
                        utils::link(widget::text!("Budget: {}", budget.name()))
                            .on_press(Message::ViewBudget(*budget.id())),
                        widget::checkbox(
                            "Negative",
                            transaction
                                .budget()
                                .map_or(false, |x| x.1 == fm_core::Sign::Negative)
                        )
                    ]
                    .spacing(10),
                );
            }

            if let Some(content) = transaction.description() {
                column = column.push(
                    widget::row![
                        widget::text("Description: "),
                        widget::container(widget::text(content.to_string()))
                            .padding(3)
                            .style(utils::style::container_style_background_weak)
                    ]
                    .spacing(10),
                );
            }

            let mut category_column = widget::Column::new().spacing(10);
            for category in transaction.categories() {
                category_column = category_column.push(
                    widget::row![
                        widget::checkbox(
                            categories
                                .iter()
                                .find(|x| x.id() == category.0)
                                .unwrap()
                                .name(),
                            true,
                        ),
                        widget::checkbox("Negative", *category.1 == fm_core::Sign::Negative)
                    ]
                    .spacing(10),
                );
            }

            widget::column![
                utils::heading("Transaction", utils::HeadingLevel::H1),
                widget::row![
                    column,
                    widget::Space::with_width(iced::Length::Fill),
                    widget::column![
                        widget::button("Edit").on_press(Message::Edit),
                        widget::button("Delete")
                            .on_press(Message::Delete)
                            .style(widget::button::danger),
                        widget::button("New Bill")
                            .on_press(Message::NewBill)
                            .style(widget::button::secondary)
                    ]
                    .spacing(10)
                ],
                widget::horizontal_rule(10),
                widget::scrollable(category_column)
            ]
            .into()
        } else {
            widget::text("Loading...").into()
        }
    }
}
