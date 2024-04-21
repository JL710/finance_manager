use fm_core;

use iced::widget;

use super::super::utils;
use super::super::{AppMessage, View};

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move { finance_manager.lock().await.get_budgets().await },
        |budgets| {
            AppMessage::SwitchView(View::CreateTransactionView(
                super::create_transaction::CreateTransactionView::new(budgets),
            ))
        },
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    AmountInput(String),
    TitleInput(String),
    DescriptionInput(String),
    DateInput(String),
    SourceInput(String),
    BudgetSelected(fm_core::Budget),
    Submit,
}

#[derive(Debug, Clone)]
pub struct CreateTransactionView {
    amount_input: String,
    title_input: String,
    description_input: String,
    source_input: String,
    destination_input: String,
    budget_state: widget::combo_box::State<fm_core::Budget>,
    date_input: String,
}

impl CreateTransactionView {
    pub fn new(budgets: Vec<fm_core::Budget>) -> Self {
        Self {
            amount_input: String::new(),
            title_input: String::new(),
            description_input: String::new(),
            source_input: String::new(),
            destination_input: String::new(),
            budget_state: widget::combo_box::State::new(budgets),
            date_input: String::new(),
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Submit => {
                todo!()
            }
            Message::AmountInput(content) => {
                self.amount_input = content;
            }
            Message::TitleInput(content) => self.title_input = content,
            Message::DescriptionInput(content) => self.description_input = content,
            Message::DateInput(content) => self.date_input = content,
            Message::SourceInput(content) => self.source_input = content,
            Message::BudgetSelected(_) => {}
        }
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        widget::column![
            utils::labeled_entry("Amount", &self.amount_input, Message::AmountInput),
            utils::labeled_entry("Title", &self.title_input, Message::TitleInput),
            utils::labeled_entry(
                "Description",
                &self.description_input,
                Message::DescriptionInput
            ),
            utils::labeled_entry("Date", &self.date_input, Message::DateInput),
            widget::row![
                widget::text("Source"),
                widget::text_input("Source", &self.source_input).on_input(Message::SourceInput)
            ]
            .spacing(10),
            widget::row![
                widget::text("Budget"),
                widget::ComboBox::new(&self.budget_state, "Budget", None, Message::BudgetSelected)
            ]
            .spacing(10),
            widget::button("Submit").on_press_maybe(if self.submittable() {
                Some(Message::Submit)
            } else {
                None
            })
        ]
        .spacing(10)
        .into()
    }

    fn submittable(&self) -> bool {
        // check if title is given
        if self.title_input.is_empty() {
            return false;
        }
        // check if amount is a valid number
        if self.amount_input.parse::<f64>().is_err() {
            return false;
        }
        // check if date is empty
        if self.date_input.is_empty() || utils::parse_to_datetime(&self.date_input).is_err() {
            return false;
        }
        // check if source and destination are empty
        if self.source_input.is_empty() && self.destination_input.is_empty() {
            return false;
        }
        true
    }
}
