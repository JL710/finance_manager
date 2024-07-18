use super::super::{utils, AppMessage, View};
use anyhow::Result;
use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(Budget::fetch(id, 0, finance_manager), |result| {
        AppMessage::SwitchView(View::ViewBudgetView(result.unwrap()))
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    Edit,
    IncreaseOffset,
    DecreaseOffset,
}

#[derive(Debug, Clone)]
pub struct Budget {
    budget: fm_core::Budget,
    current_value: fm_core::Currency,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    offset: i32,
    time_span: fm_core::Timespan,
}

impl Budget {
    pub fn new(
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        offset: i32,
    ) -> Self {
        let timespan = fm_core::calculate_budget_timespan(&budget, offset, chrono::Utc::now());
        Self {
            budget,
            current_value,
            transactions,
            offset,
            time_span: timespan,
        }
    }

    pub async fn fetch(
        id: fm_core::Id,
        offset: i32,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let budget = locked_manager.get_budget(id).await?.unwrap();
        let transactions = locked_manager
            .get_budget_transactions(&budget, offset)
            .await?;
        let mut current_value = fm_core::Currency::Eur(0.0);
        for transaction in &transactions {
            current_value += transaction.amount();
        }

        let mut transaction_tuples = Vec::new();
        for transaction in transactions {
            let source = locked_manager
                .get_account(*transaction.source())
                .await?
                .unwrap();
            let destination = locked_manager
                .get_account(*transaction.destination())
                .await?
                .unwrap();
            transaction_tuples.push((transaction, source, destination));
        }

        Ok(Self::new(budget, current_value, transaction_tuples, offset))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::ViewAccount(id) => (
                Some(View::Empty),
                super::account::switch_view_command(id, finance_manager),
            ),
            Message::ViewTransaction(id) => (
                Some(View::Empty),
                super::transaction::switch_view_command(id, finance_manager),
            ),
            Message::Edit => (
                Some(View::Empty),
                super::create_budget::switch_view_command_edit(*self.budget.id(), finance_manager),
            ),
            Message::IncreaseOffset => {
                self.offset += 1;
                self.time_span = fm_core::calculate_budget_timespan(
                    &self.budget,
                    self.offset,
                    chrono::Utc::now(),
                );
                (
                    None,
                    iced::Task::perform(
                        Self::fetch(*self.budget.id(), self.offset, finance_manager),
                        |result| AppMessage::SwitchView(View::ViewBudgetView(result.unwrap())),
                    ),
                )
            }
            Message::DecreaseOffset => {
                self.offset -= 1;
                self.time_span = fm_core::calculate_budget_timespan(
                    &self.budget,
                    self.offset,
                    chrono::Utc::now(),
                );
                (
                    None,
                    iced::Task::perform(
                        Self::fetch(*self.budget.id(), self.offset, finance_manager),
                        |result| AppMessage::SwitchView(View::ViewBudgetView(result.unwrap())),
                    ),
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let mut column = widget::column![
            widget::row![
                widget::button("<").on_press(Message::DecreaseOffset),
                widget::text!("Offset: {}", self.offset),
                widget::text!(
                    "Time Span: {} - {}",
                    self.time_span.0.unwrap().format("%d.%m.%Y").to_string(),
                    self.time_span.1.unwrap().format("%d.%m.%Y").to_string()
                ),
                widget::button(">").on_press(Message::IncreaseOffset),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(10),
            widget::text!("Name: {}", self.budget.name()),
            widget::text!("Total Value: {}", self.budget.total_value()),
            widget::text!("Current Value: {}", self.current_value),
            widget::text!("Recouring: {}", self.budget.timespan())
        ]
        .spacing(10);

        if let Some(content) = self.budget.description() {
            column = column.push(widget::text!("Description: {}", content));
        }

        widget::column![
            utils::heading("Budget", utils::HeadingLevel::H1),
            widget::row![
                column,
                widget::Space::with_width(iced::Length::Fill),
                widget::button("Edit").on_press(Message::Edit)
            ],
            widget::progress_bar(
                0.0..=self.budget.total_value().get_eur_num() as f32,
                self.current_value.get_eur_num() as f32
            ),
            utils::transaction_table(
                self.transactions.to_vec(),
                |transaction| Some(transaction.budget().unwrap().1 == fm_core::Sign::Positive),
                Message::ViewTransaction,
                Message::ViewAccount,
            )
        ]
        .spacing(10)
        .into()
    }
}
