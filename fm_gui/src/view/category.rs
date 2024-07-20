use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    category_id: fm_core::Id,
) -> iced::Task<AppMessage> {
    let (view, task) = Category::fetch(finance_manager.clone(), category_id);
    iced::Task::done(AppMessage::SwitchView(View::ViewCategory(view)))
        .chain(task.map(AppMessage::ViewCategoryMessage))
}

pub enum Action {
    None,
    EditCategory(fm_core::Id),
    DeleteCategory(iced::Task<()>),
    Task(iced::Task<Message>),
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    Delete,
    Edit,
    ChangedTimespan(fm_core::Timespan),
    Set(
        fm_core::Category,
        fm_core::Timespan,
        Vec<(fm_core::DateTime, fm_core::Currency)>,
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Category {
    NotLoaded,
    Loaded {
        category: fm_core::Category,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        timespan: fm_core::Timespan,
        values: Vec<(fm_core::DateTime, fm_core::Currency)>,
    },
}

impl Category {
    pub fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        category_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let transactions = locked_manager
                    .get_transactions_of_category(category_id, (None, None))
                    .await
                    .unwrap();
                let accounts = locked_manager.get_accounts_hash_map().await.unwrap();
                let mut transaction_tuples = Vec::new();
                for transaction in transactions {
                    let from_account = accounts.get(transaction.source()).unwrap().clone();
                    let to_account = accounts.get(transaction.destination()).unwrap().clone();
                    transaction_tuples.push((transaction, from_account, to_account));
                }
                let values = locked_manager
                    .get_relative_category_values(category_id, (None, None))
                    .await
                    .unwrap();
                let category = locked_manager
                    .get_category(category_id)
                    .await
                    .unwrap()
                    .unwrap();

                Message::Set(category, (None, None), values, transaction_tuples)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::Delete => {
                if let Self::Loaded { category, .. } = self {
                    let category_id = *category.id();
                    Action::DeleteCategory(iced::Task::future(async move {
                        finance_manager
                            .lock()
                            .await
                            .delete_category(category_id)
                            .await
                            .unwrap();
                        ()
                    }))
                } else {
                    Action::None
                }
            }
            Message::Edit => {
                if let Self::Loaded { category, .. } = self {
                    Action::EditCategory(*category.id())
                } else {
                    Action::None
                }
            }
            Message::ChangedTimespan(new_timespan) => {
                if let Self::Loaded { category, .. } = self {
                    let cloned_category = category.clone();
                    let id = *category.id();
                    Action::Task(iced::Task::future(async move {
                        let transactions = finance_manager
                            .lock()
                            .await
                            .get_transactions_of_category(id, new_timespan)
                            .await
                            .unwrap();
                        let accounts = finance_manager
                            .lock()
                            .await
                            .get_accounts_hash_map()
                            .await
                            .unwrap();
                        let mut transaction_tuples = Vec::new();
                        for transaction in transactions {
                            let from_account = accounts.get(transaction.source()).unwrap().clone();
                            let to_account =
                                accounts.get(transaction.destination()).unwrap().clone();
                            transaction_tuples.push((transaction, from_account, to_account));
                        }
                        let values = finance_manager
                            .lock()
                            .await
                            .get_relative_category_values(id, new_timespan)
                            .await
                            .unwrap();
                        Message::Set(cloned_category, new_timespan, values, transaction_tuples)
                    }))
                } else {
                    Action::None
                }
            }
            Message::Set(category, timespan, values, transactions) => {
                if let Self::Loaded { .. } = self {
                    *self = Self::Loaded {
                        category,
                        timespan,
                        transactions,
                        values,
                    };
                }
                Action::None
            }
            Message::ViewTransaction(transaction_id) => Action::ViewTransaction(transaction_id),
            Message::ViewAccount(account_id) => Action::ViewAccount(account_id),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if let Self::Loaded {
            category,
            transactions,
            values,
            ..
        } = self
        {
            widget::column![
                utils::heading("Category", utils::HeadingLevel::H1),
                widget::row![
                    widget::column![
                        widget::row![
                            widget::text("Total value"),
                            widget::text(if let Some(value) = values.last() {
                                value.1.to_string()
                            } else {
                                "0€".to_string()
                            }),
                        ]
                        .spacing(10),
                        widget::text(category.name().to_string()),
                    ]
                    .spacing(10),
                    widget::Space::with_width(iced::Length::Fill),
                    widget::column![
                        widget::button("Edit").on_press(Message::Edit),
                        widget::button("Delete").on_press(Message::Delete),
                    ]
                    .spacing(10)
                ]
                .spacing(10),
                utils::TimespanInput::new(Message::ChangedTimespan, None).into_element(),
                utils::transaction_table(
                    transactions.clone(),
                    |_| None,
                    Message::ViewTransaction,
                    Message::ViewAccount,
                )
            ]
            .spacing(10)
            .width(iced::Length::Fill)
            .into()
        } else {
            widget::text("Loading...").into()
        }
    }
}
