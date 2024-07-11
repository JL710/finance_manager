use anyhow::Result;

use async_std::sync::Mutex;
use std::sync::Arc;

use iced::widget;

use super::super::{utils, AppMessage, View};

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
        async move { Transaction::fetch(id, finance_manager).await.unwrap() },
        |x| AppMessage::SwitchView(View::TransactionView(x)),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit,
    Delete,
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub struct Transaction {
    transaction: fm_core::Transaction,
    source: fm_core::account::Account,
    destination: fm_core::account::Account,
    budget: Option<fm_core::Budget>,
    categories: Vec<fm_core::Category>,
}

impl Transaction {
    pub fn new(
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        categories: Vec<fm_core::Category>,
    ) -> Self {
        Self {
            transaction,
            source,
            destination,
            budget,
            categories,
        }
    }

    pub async fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let transaction = locked_manager.get_transaction(id).await?.unwrap();
        let source = locked_manager
            .get_account(*transaction.source())
            .await?
            .unwrap();
        let destination = locked_manager
            .get_account(*transaction.destination())
            .await?
            .unwrap();
        let budget = match transaction.budget() {
            Some(budget_id) => locked_manager.get_budget(budget_id.0).await?,
            None => None,
        };
        let categories = locked_manager.get_categories().await?;
        Ok(Self::new(
            transaction,
            source,
            destination,
            budget,
            categories,
        ))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::Edit => (
                Some(View::Empty),
                super::create_transaction::edit_switch_view_command(
                    *self.transaction.id(),
                    finance_manager,
                ),
            ),
            Message::Delete => {
                let id = *self.transaction.id();
                (
                    Some(View::Empty),
                    iced::Task::perform(
                        async move {
                            finance_manager
                                .lock()
                                .await
                                .delete_transaction(id)
                                .await
                                .unwrap();
                        },
                        |_| AppMessage::SwitchView(View::Empty),
                    ),
                )
            }
            Message::ViewAccount(acc) => {
                let id = acc;
                (
                    Some(View::Empty),
                    super::account::switch_view_command(id, finance_manager),
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<'static, Message> {
        let mut column = widget::column![
            widget::row![
                widget::text("Value: "),
                utils::colored_currency_display(&self.transaction.amount())
            ],
            widget::text(format!("Name: {}", self.transaction.title())),
            widget::button(widget::text(format!("Source: {}", self.source)))
                .padding(0)
                .style(utils::style::button_link_style)
                .on_press(Message::ViewAccount(*self.source.id())),
            widget::button(widget::text(format!("Destination: {}", self.destination)))
                .padding(0)
                .style(utils::style::button_link_style)
                .on_press(Message::ViewAccount(*self.source.id())),
            widget::text(format!(
                "Date: {}",
                self.transaction.date().format("%d.%m.%Y")
            )),
        ]
        .spacing(10);

        if let Some(budget) = &self.budget {
            column = column.push(
                widget::row![
                    widget::text(format!("Budget: {}", budget.name())),
                    widget::checkbox(
                        "Negative",
                        self.transaction
                            .budget()
                            .map_or(false, |x| x.1 == fm_core::Sign::Negative)
                    )
                ]
                .spacing(10),
            );
        }

        if let Some(content) = self.transaction.description() {
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
        for category in self.transaction.categories() {
            category_column = category_column.push(
                widget::row![
                    widget::checkbox(
                        self.categories
                            .iter()
                            .find(|x| *x.id() == category.0)
                            .unwrap()
                            .name(),
                        true,
                    ),
                    widget::checkbox("Negative", category.1 == fm_core::Sign::Negative)
                ]
                .spacing(10),
            );
        }

        widget::column![
            widget::row![
                column,
                widget::Space::with_width(iced::Length::Fill),
                widget::column![
                    widget::button("Edit").on_press(Message::Edit),
                    widget::button("Delete").on_press(Message::Delete)
                ]
                .spacing(10)
            ],
            widget::horizontal_rule(10),
            widget::scrollable(category_column)
        ]
        .into()
    }
}
