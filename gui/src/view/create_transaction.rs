use anyhow::Context;
use components::date_time::date_time_input;
use fm_core;
use iced::widget;
use iced_aw::widget::LabeledFrame;
use itertools::Itertools;

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
                fm_core::account::Account::AssetAccount(acc) => write!(f, "{}", acc.name),
                fm_core::account::Account::BookCheckingAccount(acc) => write!(f, "{acc}"),
            },
            SelectedAccount::New(name) => write!(f, "{name}"),
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
    AmountInput(components::currency_input::Action),
    TitleInput(String),
    DescriptionInput(widget::text_editor::Action),
    DateInput(date_time_input::Action),
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
    ToggleMetadataEditor,
    MetadataEditor(components::key_value_editor::Message),
}

#[derive(Debug)]
pub struct View {
    id: Option<fm_core::Id>,
    amount_input: components::CurrencyInput,
    title_input: components::ValidationTextInput,
    description_input: widget::text_editor::Content,
    source_input: Option<SelectedAccount>,
    source_state: widget::combo_box::State<SelectedAccount>,
    destination_input: Option<SelectedAccount>,
    destination_state: widget::combo_box::State<SelectedAccount>,
    budget_state: widget::combo_box::State<fm_core::Budget>,
    budget_input: Option<(fm_core::Budget, fm_core::Sign)>,
    date_input: date_time_input::State,
    metadata_editor: components::key_value_editor::KeyValueEditor,
    available_categories: Vec<fm_core::Category>,
    selected_categories: Vec<(fm_core::Id, fm_core::Sign)>,
    submitted: bool,
    metadata_editor_open: bool,
}

impl View {
    pub fn new(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self {
                id: None,
                amount_input: components::CurrencyInput::default(),
                title_input: components::ValidationTextInput::new(String::default()).required(true),
                description_input: widget::text_editor::Content::new(),
                source_input: None,
                source_state: widget::combo_box::State::new(Vec::new()),
                destination_input: None,
                destination_state: widget::combo_box::State::new(Vec::new()),
                budget_state: widget::combo_box::State::new(Vec::new()),
                budget_input: None,
                date_input: date_time_input::State::default(),
                metadata_editor: components::key_value_editor::KeyValueEditor::default(),
                metadata_editor_open: false,
                selected_categories: Vec::new(),
                available_categories: Vec::new(),
                submitted: false,
            },
            error::failing_task(async move {
                let budgets = finance_controller.get_budgets().await?;
                let accounts = finance_controller.get_accounts().await?;
                let categories = finance_controller.get_categories().await?;
                Ok(Message::Initialize(Box::new((
                    budgets, accounts, categories,
                ))))
            })
            .map(MessageContainer),
        )
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        transaction_id: fm_core::Id,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::new(finance_controller.clone()).0,
            error::failing_task(async move {
                let transaction = finance_controller
                    .get_transaction(transaction_id)
                    .await?
                    .context(format!("Could not find transaction {transaction_id}"))?;
                let source = finance_controller
                    .get_account(transaction.source)
                    .await?
                    .context(format!("Could not find account {}", transaction.source))?;
                let destination = finance_controller
                    .get_account(transaction.destination)
                    .await?
                    .context(format!(
                        "Could not find account {}",
                        transaction.destination
                    ))?;
                let budget = match transaction.budget {
                    Some(x) => finance_controller.get_budget(x.0).await?,
                    None => None,
                };
                let budgets = finance_controller.get_budgets().await?;
                let accounts = finance_controller.get_accounts().await?;
                let available_categories = finance_controller.get_categories().await?;

                Ok(Message::InitializeFromExisting(Box::new(InitExisting {
                    transaction,
                    source,
                    destination,
                    budget,
                    budgets,
                    accounts,
                    available_categories,
                })))
            })
            .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> Action {
        match message.0 {
            Message::TransactionCreated(id) => return Action::TransactionCreated(id),
            Message::Submit => {
                self.submitted = true;
                return Action::Task(
                    self.submit_command(finance_controller, utc_offset)
                        .map(|x| Message::TransactionCreated(x.id))
                        .map(MessageContainer),
                );
            }
            Message::AmountInput(action) => {
                self.amount_input.perform(action);
            }
            Message::TitleInput(content) => self.title_input.edit_content(content),
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
                let (mut budgets, mut accounts, mut categories) = *init;
                budgets.sort_by(|a, b| a.name.cmp(&b.name));
                accounts.sort_by(|a, b| a.name().cmp(b.name()));
                categories.sort_by(|a, b| a.name.cmp(&b.name));
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
                self.id = Some(init_existing.transaction.id);
                self.amount_input
                    .set_value(init_existing.transaction.amount().clone());
                self.title_input
                    .set_content(init_existing.transaction.title);
                self.description_input = widget::text_editor::Content::with_text(
                    &init_existing.transaction.description.unwrap_or_default(),
                );
                self.source_input = Some(SelectedAccount::Account(init_existing.source));
                self.source_state = widget::combo_box::State::new(
                    init_existing
                        .accounts
                        .iter()
                        .sorted_by(|a, b| a.name().cmp(b.name()))
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
                self.destination_input = Some(SelectedAccount::Account(init_existing.destination));
                self.destination_state = widget::combo_box::State::new(
                    init_existing
                        .accounts
                        .iter()
                        .sorted_by(|a, b| a.name().cmp(b.name()))
                        .map(|acc| SelectedAccount::Account(acc.clone()))
                        .collect(),
                );
                self.budget_input = init_existing
                    .budget
                    .map(|x| (x, init_existing.transaction.budget.unwrap().1));
                self.budget_state = widget::combo_box::State::new(
                    init_existing
                        .budgets
                        .into_iter()
                        .sorted_by(|a, b| a.name.cmp(&b.name))
                        .collect(),
                );
                self.date_input = date_time_input::State::new(Some(
                    components::date_time::offset_to_primitive(init_existing.transaction.date),
                ));
                self.metadata_editor = components::key_value_editor::KeyValueEditor::from(
                    init_existing.transaction.metadata,
                );
                self.available_categories = init_existing
                    .available_categories
                    .into_iter()
                    .sorted_by(|a, b| a.name.cmp(&b.name))
                    .collect();
                self.selected_categories = init_existing
                    .transaction
                    .categories
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
            Message::ToggleMetadataEditor => {
                self.metadata_editor_open = !self.metadata_editor_open;
            }
            Message::MetadataEditor(m) => {
                self.metadata_editor.update(m);
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        if self.submitted {
            return "Loading...".into();
        }

        let editing_pane = if self.metadata_editor_open {
            iced::Element::new(components::spaced_column![
                components::button::back("Back", Some(Message::ToggleMetadataEditor)),
                self.metadata_editor.view().map(Message::MetadataEditor)
            ])
        } else {
            let mut categories = components::spaced_column![];
            for category in &self.available_categories {
                let selected = self.selected_categories.iter().find(|x| x.0 == category.id);
                categories = categories.push(components::spal_row![
                    widget::checkbox(&category.name, selected.is_some())
                        .on_toggle(move |_| { Message::SelectCategory(category.id) }),
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
                            category.id,
                            if s.1 == fm_core::Sign::Negative {
                                fm_core::Sign::Positive
                            } else {
                                fm_core::Sign::Negative
                            },
                        )
                    }))
                ]);
            }

            let source_acc_style = if let Some(acc) = &self.source_input {
                match acc {
                    SelectedAccount::Account(_) => style::text_input_success,
                    SelectedAccount::New(_) => style::text_input_primary,
                }
            } else {
                style::text_input_danger
            };
            let destination_acc_style = if let Some(acc) = &self.destination_input {
                match acc {
                    SelectedAccount::Account(_) => style::text_input_success,
                    SelectedAccount::New(_) => style::text_input_primary,
                }
            } else {
                style::text_input_danger
            };

            iced::Element::new(components::spaced_column![
                components::spal_row![
                    "Amount: ",
                    self.amount_input.view().map(Message::AmountInput),
                ]
                .width(iced::Fill),
                components::labeled_entry(
                    "Title",
                    "",
                    &self.title_input,
                    Some(Message::TitleInput)
                ),
                components::spaced_row![
                    "Description",
                    widget::text_editor(&self.description_input)
                        .on_action(Message::DescriptionInput)
                ],
                components::spal_row![
                    "Date: ",
                    date_time_input::date_time_input(&self.date_input, true)
                        .view()
                        .map(Message::DateInput)
                ]
                .width(iced::Fill),
                components::spal_row![
                    "Source",
                    widget::ComboBox::new(
                        &self.source_state,
                        "Source",
                        self.source_input.as_ref(),
                        Message::SourceSelected
                    )
                    .on_input(Message::SourceInput)
                    .input_style(source_acc_style)
                ],
                components::spal_row![
                    "Destination",
                    widget::ComboBox::new(
                        &self.destination_state,
                        "Destination",
                        self.destination_input.as_ref(),
                        Message::DestinationSelected
                    )
                    .on_input(Message::DestinationInput)
                    .input_style(destination_acc_style)
                ],
                components::spal_row![
                    "Budget",
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
                .align_y(iced::Center),
                LabeledFrame::new("Categories", categories).width(iced::Fill),
                widget::button("Metadata").on_press(Message::ToggleMetadataEditor)
            ])
        };

        iced::Element::new(widget::scrollable(components::spaced_column![
            editing_pane,
            widget::horizontal_rule(10),
            components::submit_cancel_row(
                if self.submittable() {
                    Some(Message::Submit)
                } else {
                    None
                },
                Some(Message::Cancel)
            ),
        ]))
        .map(MessageContainer)
    }

    fn submittable(&self) -> bool {
        // check if title is given
        if !self.title_input.is_valid() {
            return false;
        }
        // check if amount is a valid currency
        if self.amount_input.currency().is_none() {
            return false;
        }
        // check if date is empty
        if self.date_input.datetime().is_none() {
            return false;
        }
        // check if source and destination are empty
        if self.source_input.is_none() && self.destination_input.is_none() {
            return false;
        }
        // check if source and destination are valid
        if let Some(source_input) = &self.source_input
            && let Some(destination_input) = &self.destination_input
        {
            // check if both are new
            if source_input.is_new() && destination_input.is_new() {
                return false;
            }
            if source_input == destination_input {
                return false;
            }
        }
        true
    }

    fn submit_command(
        &self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<fm_core::Transaction> {
        let option_id = self.id;
        let amount = self.amount_input.currency().unwrap();
        let title = self.title_input.value().clone();
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
            .map(|budget| (budget.0.id, budget.1));
        let date = self.date_input.datetime().unwrap();
        let metadata = std::collections::HashMap::from_iter(self.metadata_editor.pairs());
        let mut categories =
            std::collections::HashMap::with_capacity(self.selected_categories.len());
        for (id, sign) in &self.selected_categories {
            categories.insert(*id, *sign);
        }
        error::failing_task(async move {
            let source_id = match source {
                SelectedAccount::Account(acc) => *acc.id(),
                SelectedAccount::New(name) => {
                    finance_controller
                        .create_book_checking_account(name, None, None, None)
                        .await?
                        .id
                }
            };

            let destination_id = match destination {
                SelectedAccount::Account(acc) => *acc.id(),
                SelectedAccount::New(name) => {
                    finance_controller
                        .create_book_checking_account(name, None, None, None)
                        .await?
                        .id
                }
            };

            Ok(match option_id {
                Some(id) => {
                    finance_controller
                        .update_transaction(fm_core::Transaction::new(
                            id,
                            amount,
                            title,
                            description,
                            source_id,
                            destination_id,
                            budget,
                            components::date_time::primitive_to_offset(date, utc_offset),
                            metadata,
                            categories,
                        )?)
                        .await?
                }
                _ => {
                    finance_controller
                        .create_transaction(
                            amount,
                            title,
                            description,
                            source_id,
                            destination_id,
                            budget,
                            components::date_time::primitive_to_offset(date, utc_offset),
                            metadata,
                            categories,
                        )
                        .await?
                }
            })
        })
    }
}
