use fm_core;

use iced::widget;

use super::super::utils;
use super::super::{AppMessage, View};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            let budgets = finance_manager.lock().await.get_budgets().await.unwrap();
            let accounts = finance_manager.lock().await.get_accounts().await.unwrap();
            let categories = finance_manager.lock().await.get_categories().await.unwrap();
            (budgets, accounts, categories)
        },
        |x| {
            AppMessage::SwitchView(View::CreateTransactionView(
                super::create_transaction::CreateTransactionView::new(x.0, x.1, x.2),
            ))
        },
    )
}

pub fn edit_switch_view_command(
    id: fm_core::Id,
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            CreateTransactionView::fetch(&id, finance_manager)
                .await
                .unwrap()
        },
        |x| AppMessage::SwitchView(View::CreateTransactionView(x)),
    )
}

#[derive(Debug, Clone, PartialEq)]
enum SelectedAccount {
    Account(fm_core::account::Account),
    New(String),
}

impl SelectedAccount {
    fn is_new(&self) -> bool {
        matches!(self, SelectedAccount::New(_))
    }
}

impl std::fmt::Display for SelectedAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectedAccount::Account(account) => match account {
                fm_core::account::Account::AssetAccount(acc) => write!(f, "{}", acc.name()),
                fm_core::account::Account::BookCheckingAccount(acc) => write!(f, "{}", acc),
            },
            SelectedAccount::New(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AmountInput(String),
    TitleInput(String),
    DescriptionInput(String),
    DateInput(String),
    SourceInput(String),
    SourceSelected(SelectedAccount),
    DestinationInput(String),
    DestinationSelected(SelectedAccount),
    BudgetSelected(fm_core::Budget),
    ClearBudget,
    Submit,
    SelectCategory(fm_core::Id),
}

#[derive(Debug, Clone)]
pub struct CreateTransactionView {
    id: Option<fm_core::Id>,
    amount_input: String,
    title_input: String,
    description_input: String,
    source_input: Option<SelectedAccount>,
    source_state: widget::combo_box::State<SelectedAccount>,
    destination_input: Option<SelectedAccount>,
    destination_state: widget::combo_box::State<SelectedAccount>,
    budget_state: widget::combo_box::State<fm_core::Budget>,
    budget_input: Option<fm_core::Budget>,
    date_input: String,
    metadata: std::collections::HashMap<String, String>,
    available_categories: Vec<fm_core::Category>,
    selected_categories: Vec<fm_core::Id>,
}

impl CreateTransactionView {
    pub fn new(
        budgets: Vec<fm_core::Budget>,
        accounts: Vec<fm_core::account::Account>,
        available_categories: Vec<fm_core::Category>,
    ) -> Self {
        Self {
            id: None,
            amount_input: String::new(),
            title_input: String::new(),
            description_input: String::new(),
            source_input: None,
            source_state: widget::combo_box::State::new(
                accounts
                    .iter()
                    .map(|acc| SelectedAccount::Account(acc.clone()))
                    .collect(),
            ),
            destination_input: None,
            destination_state: widget::combo_box::State::new(
                accounts
                    .iter()
                    .map(|acc| SelectedAccount::Account(acc.clone()))
                    .collect(),
            ),
            budget_state: widget::combo_box::State::new(budgets),
            budget_input: None,
            date_input: String::new(),
            metadata: std::collections::HashMap::new(),
            selected_categories: Vec::new(),
            available_categories,
        }
    }

    pub fn from_existing_transaction(
        transaction: &fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        budgets: Vec<fm_core::Budget>,
        accounts: Vec<fm_core::account::Account>,
        available_categories: Vec<fm_core::Category>,
    ) -> Self {
        fn string_convert(input: Option<&str>) -> String {
            match input {
                Some(x) => x.to_owned(),
                _ => String::new(),
            }
        }

        Self {
            id: Some(*transaction.id()),
            amount_input: transaction.amount().get_num().to_string(),
            title_input: transaction.title().clone(),
            description_input: string_convert(transaction.description()),
            source_input: Some(SelectedAccount::Account(source)),
            source_state: widget::combo_box::State::new(
                accounts
                    .iter()
                    .map(|acc| SelectedAccount::Account(acc.clone()))
                    .collect(),
            ),
            destination_input: Some(SelectedAccount::Account(destination)),
            destination_state: widget::combo_box::State::new(
                accounts
                    .iter()
                    .map(|acc| SelectedAccount::Account(acc.clone()))
                    .collect(),
            ),
            budget_input: budget,
            budget_state: widget::combo_box::State::new(budgets),
            date_input: transaction.date().format("%d.%m.%Y").to_string(),
            metadata: transaction.metadata().clone(),
            available_categories,
            selected_categories: transaction.categories().to_vec(),
        }
    }

    pub async fn fetch(
        id: &fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;

        let transaction = locked_manager.get_transaction(*id).await?.unwrap();
        let source_account = locked_manager
            .get_account(*transaction.source())
            .await?
            .unwrap();
        let destination_account = locked_manager
            .get_account(*transaction.destination())
            .await?
            .unwrap();
        let budget = match transaction.budget() {
            Some(x) => locked_manager.get_budget(*x).await?,
            None => None,
        };
        let budgets = locked_manager.get_budgets().await?;
        let accounts = locked_manager.get_accounts().await?;
        let available_categories = locked_manager.get_categories().await?;

        Ok(Self::from_existing_transaction(
            &transaction,
            source_account,
            destination_account,
            budget,
            budgets,
            accounts,
            available_categories,
        ))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::Submit => {
                return (Some(View::Empty), self.submit_command(finance_manager));
            }
            Message::AmountInput(content) => {
                self.amount_input = content;
            }
            Message::TitleInput(content) => self.title_input = content,
            Message::DescriptionInput(content) => self.description_input = content,
            Message::DateInput(content) => self.date_input = content,
            Message::SourceInput(content) => {
                self.source_input = Some(SelectedAccount::New(content))
            }
            Message::BudgetSelected(content) => {
                self.budget_input = Some(content);
            }
            Message::SourceSelected(content) => {
                self.source_input = Some(content);
            }
            Message::DestinationInput(content) => {
                self.destination_input = Some(SelectedAccount::New(content));
            }
            Message::DestinationSelected(content) => {
                self.destination_input = Some(content);
            }
            Message::ClearBudget => {
                self.budget_input = None;
            }
            Message::SelectCategory(id) => {
                if self.selected_categories.contains(&id) {
                    self.selected_categories.retain(|x| x != &id);
                } else {
                    self.selected_categories.push(id);
                }
            }
        }
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let mut categories = widget::column![].spacing(10);
        for category in &self.available_categories {
            let selected = self.selected_categories.contains(category.id());
            categories = categories.push(
                widget::checkbox(category.name(), selected)
                    .on_toggle(|_| Message::SelectCategory(*category.id())),
            );
        }

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
                widget::ComboBox::new(
                    &self.source_state,
                    "Source",
                    self.source_input.as_ref(),
                    Message::SourceSelected
                )
                .on_input(Message::SourceInput)
            ]
            .spacing(10),
            widget::row![
                widget::text("Destination"),
                widget::ComboBox::new(
                    &self.destination_state,
                    "Destination",
                    self.destination_input.as_ref(),
                    Message::DestinationSelected
                )
                .on_input(Message::DestinationInput)
            ]
            .spacing(10),
            widget::row![
                widget::text("Budget"),
                widget::ComboBox::new(
                    &self.budget_state,
                    "Budget",
                    self.budget_input.as_ref(),
                    Message::BudgetSelected
                ),
                widget::button("X").on_press(Message::ClearBudget)
            ]
            .spacing(10),
            widget::horizontal_rule(10),
            widget::text("Categories"),
            widget::scrollable(categories)
                .height(iced::Length::Fill)
                .width(iced::Length::Fill),
            widget::horizontal_rule(10),
            widget::button("Submit").on_press_maybe(if self.submittable() {
                Some(Message::Submit)
            } else {
                None
            })
        ]
        .height(iced::Length::Fill)
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
        if self.source_input.is_none() && self.destination_input.is_none() {
            return false;
        }
        // check if source and destination are valid
        if let Some(source_input) = &self.source_input {
            if let Some(destination_input) = &self.destination_input {
                // check if both are new
                if source_input.is_new() && destination_input.is_new() {
                    return false;
                }
                if source_input == destination_input {
                    return false;
                }
            }
        }
        true
    }

    fn submit_command(
        &self,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> iced::Command<AppMessage> {
        let option_id = self.id;
        let amount = fm_core::Currency::Eur(self.amount_input.parse::<f64>().unwrap());
        let title = self.title_input.clone();
        let description = if self.description_input.is_empty() {
            None
        } else {
            Some(self.description_input.clone())
        };
        let source = match &self.source_input {
            Some(SelectedAccount::Account(acc)) => fm_core::Or::One(*acc.id()),
            Some(SelectedAccount::New(name)) => fm_core::Or::Two(name.clone()),
            None => panic!(),
        };
        let destination = match &self.destination_input {
            Some(SelectedAccount::Account(acc)) => fm_core::Or::One(*acc.id()),
            Some(SelectedAccount::New(name)) => fm_core::Or::Two(name.clone()),
            None => panic!(),
        };
        let budget = self.budget_input.as_ref().map(|budget| *budget.id());
        let date = utils::parse_to_datetime(&self.date_input).unwrap();
        let metadata = self.metadata.clone();
        let categories = self.selected_categories.clone();
        iced::Command::perform(
            async move {
                let new_transaction = match option_id {
                    Some(id) => finance_manager
                        .lock()
                        .await
                        .update_transaction(
                            id,
                            amount,
                            title,
                            description,
                            source,
                            destination,
                            budget,
                            date,
                            metadata,
                            categories,
                        )
                        .await
                        .unwrap(),
                    _ => finance_manager
                        .lock()
                        .await
                        .create_transaction(
                            amount,
                            title,
                            description,
                            source,
                            destination,
                            budget,
                            date,
                            metadata,
                            categories,
                        )
                        .await
                        .unwrap(),
                };
                super::view_transaction::TransactionView::fetch(
                    *new_transaction.id(),
                    finance_manager,
                )
                .await
                .unwrap()
            },
            |x| AppMessage::SwitchView(View::TransactionView(x)),
        )
    }
}
