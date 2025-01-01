use fm_core;

use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

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

pub enum Action {
    None,
    TransactionCreated(fm_core::Id),
    Task(iced::Task<MessageContainer>),
    Cancel,
    CancelWithId(fm_core::Id),
}

#[derive(Debug, Clone)]
struct InitExisting {
    transaction: fm_core::Transaction,
    source: fm_core::account::Account,
    destination: fm_core::account::Account,
    budget: Option<fm_core::Budget>,
    budgets: Vec<fm_core::Budget>,
    accounts: Vec<fm_core::account::Account>,
    available_categories: Vec<fm_core::Category>,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    AmountInput(utils::currency_input::Action),
    TitleInput(String),
    DescriptionInput(widget::text_editor::Action),
    DateInput(utils::date_input::Action),
    SourceInput(String),
    SourceSelected(SelectedAccount),
    DestinationInput(String),
    DestinationSelected(SelectedAccount),
    BudgetSelected(fm_core::Budget),
    BudgetSignChange(bool),
    ClearBudget,
    Submit,
    SelectCategory(fm_core::Id),
    ChangeSelectedCategorySign(fm_core::Id, fm_core::Sign),
    Initialize(
        Box<(
            Vec<fm_core::Budget>,
            Vec<fm_core::account::Account>,
            Vec<fm_core::Category>,
        )>,
    ),
    InitializeFromExisting(Box<InitExisting>),
    TransactionCreated(fm_core::Id),
    Cancel,
}

#[derive(Debug)]
pub struct CreateTransactionView {
    id: Option<fm_core::Id>,
    amount_input: utils::currency_input::State,
    title_input: String,
    description_input: widget::text_editor::Content,
    source_input: Option<SelectedAccount>,
    source_state: widget::combo_box::State<SelectedAccount>,
    destination_input: Option<SelectedAccount>,
    destination_state: widget::combo_box::State<SelectedAccount>,
    budget_state: widget::combo_box::State<fm_core::Budget>,
    budget_input: Option<(fm_core::Budget, fm_core::Sign)>,
    date_input: utils::date_input::State,
    metadata: std::collections::HashMap<String, String>,
    available_categories: Vec<fm_core::Category>,
    selected_categories: Vec<(fm_core::Id, fm_core::Sign)>,
    submitted: bool,
}

impl CreateTransactionView {
    pub fn new(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self {
                id: None,
                amount_input: utils::currency_input::State::default(),
                title_input: String::new(),
                description_input: widget::text_editor::Content::new(),
                source_input: None,
                source_state: widget::combo_box::State::new(Vec::new()),
                destination_input: None,
                destination_state: widget::combo_box::State::new(Vec::new()),
                budget_state: widget::combo_box::State::new(Vec::new()),
                budget_input: None,
                date_input: utils::date_input::State::default(),
                metadata: std::collections::HashMap::new(),
                selected_categories: Vec::new(),
                available_categories: Vec::new(),
                submitted: false,
            },
            iced::Task::future(async move {
                let budgets = finance_manager.lock().await.get_budgets().await.unwrap();
                let accounts = finance_manager.lock().await.get_accounts().await.unwrap();
                let categories = finance_manager.lock().await.get_categories().await.unwrap();
                Message::Initialize(Box::new((budgets, accounts, categories)))
            })
            .map(MessageContainer),
        )
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
        transaction_id: fm_core::Id,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::new(finance_manager.clone()).0,
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;

                let transaction = locked_manager
                    .get_transaction(transaction_id)
                    .await
                    .unwrap()
                    .unwrap();
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
                    Some(x) => locked_manager.get_budget(x.0).await.unwrap(),
                    None => None,
                };
                let budgets = locked_manager.get_budgets().await.unwrap();
                let accounts = locked_manager.get_accounts().await.unwrap();
                let available_categories = locked_manager.get_categories().await.unwrap();

                Message::InitializeFromExisting(Box::new(InitExisting {
                    transaction,
                    source,
                    destination,
                    budget,
                    budgets,
                    accounts,
                    available_categories,
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
            Message::TransactionCreated(id) => return Action::TransactionCreated(id),
            Message::Submit => {
                self.submitted = true;
                return Action::Task(
                    self.submit_command(finance_manager)
                        .map(|x| Message::TransactionCreated(*x.id()))
                        .map(MessageContainer),
                );
            }
            Message::AmountInput(action) => {
                self.amount_input.perform(action);
            }
            Message::TitleInput(content) => self.title_input = content,
            Message::DescriptionInput(action) => self.description_input.perform(action),
            Message::DateInput(action) => self.date_input.perform(action),
            Message::SourceInput(content) => {
                self.source_input = Some(SelectedAccount::New(content))
            }
            Message::BudgetSelected(content) => {
                self.budget_input = Some((
                    content,
                    self.budget_input
                        .as_ref()
                        .map_or(fm_core::Sign::Positive, |x| x.1),
                ));
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
                if self
                    .selected_categories
                    .iter()
                    .map(|x| x.0)
                    .collect::<Vec<_>>()
                    .contains(&id)
                {
                    self.selected_categories.retain(|x| x.0 != id);
                } else {
                    self.selected_categories.push((id, fm_core::Sign::Positive));
                }
            }
            Message::ChangeSelectedCategorySign(id, sign) => {
                if let Some(x) = self.selected_categories.iter_mut().find(|x| x.0 == id) {
                    x.1 = sign;
                }
            }
            Message::BudgetSignChange(x) => {
                if let Some(budget) = &self.budget_input {
                    self.budget_input = Some((
                        budget.0.clone(),
                        if x {
                            fm_core::Sign::Negative
                        } else {
                            fm_core::Sign::Positive
                        },
                    ));
                }
            }
            Message::Initialize(init) => {
                let (budgets, accounts, categories) = *init;
                self.budget_state = widget::combo_box::State::new(budgets);
                self.available_categories = categories;
                self.source_state = widget::combo_box::State::new(
                    accounts
                        .iter()
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
                self.destination_state = widget::combo_box::State::new(
                    accounts
                        .iter()
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
            }
            Message::InitializeFromExisting(init) => {
                let init_existing = *init;
                self.id = Some(*init_existing.transaction.id());
                self.amount_input =
                    utils::currency_input::State::new(init_existing.transaction.amount());
                self.title_input
                    .clone_from(init_existing.transaction.title());
                self.description_input = widget::text_editor::Content::with_text(
                    init_existing.transaction.description().unwrap_or_default(),
                );
                self.source_input = Some(SelectedAccount::Account(init_existing.source));
                self.source_state = widget::combo_box::State::new(
                    init_existing
                        .accounts
                        .iter()
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
                self.destination_input = Some(SelectedAccount::Account(init_existing.destination));
                self.destination_state = widget::combo_box::State::new(
                    init_existing
                        .accounts
                        .iter()
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
                self.budget_input = init_existing
                    .budget
                    .map(|x| (x, init_existing.transaction.budget().unwrap().1));
                self.budget_state = widget::combo_box::State::new(init_existing.budgets);
                self.date_input =
                    utils::date_input::State::new(Some(*init_existing.transaction.date()));
                self.metadata
                    .clone_from(init_existing.transaction.metadata());
                self.available_categories = init_existing.available_categories;
                self.selected_categories = init_existing
                    .transaction
                    .categories()
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .collect::<Vec<_>>();
            }
            Message::Cancel => {
                if let Some(id) = self.id {
                    return Action::CancelWithId(id);
                } else {
                    return Action::Cancel;
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        if self.submitted {
            return "Loading...".into();
        }

        let mut categories = widget::column![].spacing(10);
        for category in &self.available_categories {
            let selected = self
                .selected_categories
                .iter()
                .find(|x| x.0 == *category.id());
            categories = categories.push(
                widget::row![
                    widget::checkbox(category.name(), selected.is_some())
                        .on_toggle(move |_| { Message::SelectCategory(*category.id()) }),
                    widget::checkbox(
                        "Negative",
                        if let Some(s) = selected {
                            s.1 == fm_core::Sign::Negative
                        } else {
                            false
                        }
                    )
                    .on_toggle_maybe(selected.map(|s| |_| {
                        Message::ChangeSelectedCategorySign(
                            *category.id(),
                            if s.1 == fm_core::Sign::Negative {
                                fm_core::Sign::Positive
                            } else {
                                fm_core::Sign::Negative
                            },
                        )
                    }))
                ]
                .spacing(10),
            );
        }

        let source_acc_style = if let Some(acc) = &self.source_input {
            match acc {
                SelectedAccount::Account(_) => utils::style::text_input_success,
                SelectedAccount::New(_) => utils::style::text_input_primary,
            }
        } else {
            utils::style::text_input_danger
        };
        let destination_acc_style = if let Some(acc) = &self.destination_input {
            match acc {
                SelectedAccount::Account(_) => utils::style::text_input_success,
                SelectedAccount::New(_) => utils::style::text_input_primary,
            }
        } else {
            utils::style::text_input_danger
        };

        iced::Element::new(
            widget::column![
                utils::heading("Create Transaction", utils::HeadingLevel::H1),
                widget::row![
                    "Amount: ",
                    utils::currency_input::currency_input(&self.amount_input, true)
                        .view()
                        .map(Message::AmountInput),
                ]
                .width(iced::Fill)
                .spacing(10),
                utils::labeled_entry("Title", &self.title_input, Message::TitleInput, true),
                widget::row![
                    "Description",
                    widget::text_editor(&self.description_input)
                        .on_action(Message::DescriptionInput)
                ]
                .align_y(iced::Center)
                .spacing(10),
                widget::row![
                    "Date: ",
                    utils::date_input::date_input(&self.date_input, "", true)
                        .view()
                        .map(Message::DateInput)
                ]
                .width(iced::Fill),
                widget::row![
                    widget::text("Source"),
                    widget::ComboBox::new(
                        &self.source_state,
                        "Source",
                        self.source_input.as_ref(),
                        Message::SourceSelected
                    )
                    .on_input(Message::SourceInput)
                    .input_style(source_acc_style)
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
                    .input_style(destination_acc_style)
                ]
                .spacing(10),
                widget::row![
                    widget::text("Budget"),
                    widget::ComboBox::new(
                        &self.budget_state,
                        "Budget",
                        self.budget_input.as_ref().map(|x| &x.0),
                        Message::BudgetSelected
                    ),
                    widget::checkbox(
                        "Negative",
                        self.budget_input
                            .as_ref()
                            .is_some_and(|x| x.1 == fm_core::Sign::Negative)
                    )
                    .on_toggle_maybe(if self.budget_input.is_some() {
                        Some(Message::BudgetSignChange)
                    } else {
                        None
                    }),
                    widget::button("X").on_press(Message::ClearBudget)
                ]
                .align_y(iced::Center)
                .spacing(10),
                widget::horizontal_rule(10),
                widget::text("Categories"),
                widget::scrollable(categories)
                    .height(iced::Length::Fill)
                    .width(iced::Length::Fill),
                widget::horizontal_rule(10),
                widget::row![
                    widget::button("Cancel")
                        .on_press(Message::Cancel)
                        .style(widget::button::danger),
                    widget::horizontal_space(),
                    widget::button("Submit")
                        .on_press_maybe(if self.submittable() {
                            Some(Message::Submit)
                        } else {
                            None
                        })
                        .style(widget::button::success)
                ],
            ]
            .height(iced::Length::Fill)
            .spacing(10),
        )
        .map(MessageContainer)
    }

    fn submittable(&self) -> bool {
        // check if title is given
        if self.title_input.is_empty() {
            return false;
        }
        // check if amount is a valid currency
        if self.amount_input.currency().is_none() {
            return false;
        }
        // check if date is empty
        if self.date_input.date().is_none() {
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
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> iced::Task<fm_core::Transaction> {
        let option_id = self.id;
        let amount = self.amount_input.currency().unwrap();
        let title = self.title_input.clone();
        let description = if self.description_input.text().trim().is_empty() {
            None
        } else {
            Some(self.description_input.text())
        };
        let source = self.source_input.clone().unwrap();
        let destination = self.destination_input.clone().unwrap();
        let budget = self
            .budget_input
            .as_ref()
            .map(|budget| (*budget.0.id(), budget.1));
        let date = self.date_input.date().unwrap();
        let metadata = self.metadata.clone();
        let mut categories =
            std::collections::HashMap::with_capacity(self.selected_categories.len());
        for (id, sign) in &self.selected_categories {
            categories.insert(*id, *sign);
        }
        iced::Task::future(async move {
            let source_id = match source {
                SelectedAccount::Account(acc) => *acc.id(),
                SelectedAccount::New(name) => finance_manager
                    .lock()
                    .await
                    .create_book_checking_account(name, None, None, None)
                    .await
                    .unwrap()
                    .id(),
            };

            let destination_id = match destination {
                SelectedAccount::Account(acc) => *acc.id(),
                SelectedAccount::New(name) => finance_manager
                    .lock()
                    .await
                    .create_book_checking_account(name, None, None, None)
                    .await
                    .unwrap()
                    .id(),
            };

            match option_id {
                Some(id) => finance_manager
                    .lock()
                    .await
                    .update_transaction(
                        id,
                        amount,
                        title,
                        description,
                        source_id,
                        destination_id,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .unwrap()
                    .await
                    .unwrap(),
                _ => finance_manager
                    .lock()
                    .await
                    .create_transaction(
                        amount,
                        title,
                        description,
                        source_id,
                        destination_id,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .unwrap()
                    .await
                    .unwrap(),
            }
        })
    }
}
