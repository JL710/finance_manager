use fm_core;

use iced::widget;

use super::super::utils;
use super::super::AppMessage;

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

impl super::View for CreateTransactionView {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        message: Self::ParentMessage,
        finance_manager: &mut fm_core::FinanceManager,
    ) -> Option<Box<dyn super::View<ParentMessage = Self::ParentMessage>>> {
        if let AppMessage::CreateTransactionViewMessage(m) = message {
            self.update(m, finance_manager);
        } else {
            panic!();
        }
        None
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        self.view().map(AppMessage::CreateTransactionViewMessage)
    }
}

impl CreateTransactionView {
    pub fn new(finance_manager: &fm_core::FinanceManager) -> Self {
        Self {
            amount_input: String::new(),
            title_input: String::new(),
            description_input: String::new(),
            source_input: String::new(),
            destination_input: String::new(),
            budget_state: widget::combo_box::State::new(finance_manager.get_budgets()),
            date_input: String::new(),
        }
    }

    pub fn update(&mut self, message: Message, _finance_manager: &fm_core::FinanceManager) {
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
        if self.date_input.is_empty() {
            return false;
        }
        // check if source and destination are empty
        if self.source_input.is_empty() && self.destination_input.is_empty() {
            return false;
        }
        true
    }
}
