use crate::utils;

use super::super::{AppMessage, View};

use anyhow::Result;
use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    category_id: fm_core::Id,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            View::ViewCategory(
                ViewCategory::fetch(finance_manager, category_id)
                    .await
                    .unwrap(),
            )
        },
        AppMessage::SwitchView,
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Delete,
    Edit,
    ChangedTimespan(fm_core::Timespan),
    Set(
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
pub struct ViewCategory {
    category: fm_core::Category,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    timespan: fm_core::Timespan,
    values: Vec<(fm_core::DateTime, fm_core::Currency)>,
}

impl ViewCategory {
    pub fn new(
        category: fm_core::Category,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        values: Vec<(fm_core::DateTime, fm_core::Currency)>,
    ) -> Self {
        Self {
            category,
            transactions,
            timespan: (None, None),
            values,
        }
    }

    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        category_id: fm_core::Id,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let transactions = locked_manager
            .get_transactions_of_category(category_id, (None, None))
            .await?;
        let accounts = locked_manager.get_accounts_hash_map().await?;
        let mut transaction_tuples = Vec::new();
        for transaction in transactions {
            let from_account = accounts.get(transaction.source()).unwrap().clone();
            let to_account = accounts.get(transaction.destination()).unwrap().clone();
            transaction_tuples.push((transaction, from_account, to_account));
        }
        let values = locked_manager
            .get_relative_category_values(category_id, (None, None))
            .await?;
        Ok(Self {
            category: locked_manager.get_category(category_id).await?.unwrap(),
            transactions: transaction_tuples,
            timespan: (None, None),
            values,
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Delete => {
                let category_id = *self.category.id();
                (
                    None,
                    iced::Command::perform(
                        async move {
                            finance_manager
                                .lock()
                                .await
                                .delete_category(category_id)
                                .await
                                .unwrap();
                            AppMessage::SwitchView(View::CategoryOverview(
                                super::category_overview::CategoryOverview::fetch(finance_manager)
                                    .await
                                    .unwrap(),
                            ))
                        },
                        |msg| msg,
                    ),
                )
            }
            Message::Edit => (
                Some(View::CreateCategory(
                    super::create_category::CreateCategory::from_existing(
                        Some(*self.category.id()),
                        self.category.name().to_string(),
                    ),
                )),
                iced::Command::none(),
            ),
            Message::ChangedTimespan(timespan) => {
                self.timespan = timespan;
                let id = *self.category.id();
                (
                    None,
                    iced::Command::perform(
                        async move {
                            let transactions = finance_manager
                                .lock()
                                .await
                                .get_transactions_of_category(id, timespan)
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
                                let from_account =
                                    accounts.get(transaction.source()).unwrap().clone();
                                let to_account =
                                    accounts.get(transaction.destination()).unwrap().clone();
                                transaction_tuples.push((transaction, from_account, to_account));
                            }
                            let values = finance_manager
                                .lock()
                                .await
                                .get_relative_category_values(id, timespan)
                                .await
                                .unwrap();
                            AppMessage::ViewCategoryMessage(Message::Set(
                                timespan,
                                values,
                                transaction_tuples,
                            ))
                        },
                        |msg| msg,
                    ),
                )
            }
            Message::Set(timespan, values, transactions) => {
                self.timespan = timespan;
                self.values = values;
                self.transactions = transactions;
                (None, iced::Command::none())
            }
            Message::ViewTransaction(transaction_id) => (
                Some(View::Empty),
                super::view_transaction::switch_view_command(transaction_id, finance_manager),
            ),
            Message::ViewAccount(account_id) => (
                Some(View::Empty),
                super::view_account::switch_view_command(account_id, finance_manager),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::column![
            widget::row![
                widget::column![
                    widget::row![
                        widget::text("Total value"),
                        widget::text(if let Some(value) = self.values.last() {
                            value.1.to_string()
                        } else {
                            "0€".to_string()
                        }),
                    ]
                    .spacing(10),
                    widget::text(self.category.name().to_string()),
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
            super::super::timespan_input::TimespanInput::new(Message::ChangedTimespan)
                .into_element(),
            utils::transaction_table(
                self.transactions.clone(),
                |_| None,
                Message::ViewTransaction,
                Message::ViewAccount,
            )
        ]
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
