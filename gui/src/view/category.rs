use anyhow::Context;
use iced::widget;
use utils::date_time::date_span_input;

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
    ChangedTimespan(date_span_input::Action),
    Set(
        fm_core::Category,
        Vec<(fm_core::DateTime, fm_core::Currency)>,
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        Vec<fm_core::Category>,
        Vec<fm_core::Budget>,
    ),
    TransactionTable(utils::transaction_table::Message),
}

#[derive(Debug)]
pub enum View {
    NotLoaded,
    Loaded {
        category: fm_core::Category,
        transaction_table: Box<utils::TransactionTable>,
        values: Vec<(fm_core::DateTime, fm_core::Currency)>,
        timespan_input: date_span_input::State,
    },
}

impl View {
    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        category_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            utils::failing_task(async move {
                let transactions = finance_controller
                    .get_transactions_of_category(category_id, (None, None))
                    .await?;
                let accounts = finance_controller
                    .get_accounts_hash_map()
                    .await
                    .context("Error while fetching accounts")?;
                let mut transaction_tuples = Vec::new();
                for transaction in transactions {
                    let from_account = accounts
                        .get(&transaction.source)
                        .context("Could not find source account")?
                        .clone();
                    let to_account = accounts
                        .get(&transaction.destination)
                        .context("Could not find destination account")?
                        .clone();
                    transaction_tuples.push((transaction, from_account, to_account));
                }
                let values = finance_controller
                    .get_relative_category_values(category_id, (None, None))
                    .await?;
                let category = finance_controller
                    .get_category(category_id)
                    .await?
                    .context(format!("Category {} not found", category_id))?;

                let categories = finance_controller.get_categories().await?;
                let budgets = finance_controller.get_budgets().await?;
                Ok(Message::Set(
                    category,
                    values,
                    transaction_tuples,
                    categories,
                    budgets,
                ))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Delete => {
                if let Self::Loaded { category, .. } = self {
                    if let rfd::MessageDialogResult::No = rfd::MessageDialog::new()
                        .set_title("Delete category")
                        .set_description(format!(
                            "Do you really want to delete {}?",
                            category.name()
                        ))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        return Action::None;
                    }

                    let category_id = *category.id();
                    Action::DeleteCategory(utils::failing_task(async move {
                        finance_controller.delete_category(category_id).await?;
                        Ok(())
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
            Message::ChangedTimespan(action) => {
                if let Self::Loaded {
                    category,
                    timespan_input,
                    ..
                } = self
                {
                    timespan_input.perform(action);

                    let cloned_category = category.clone();
                    let id = *category.id();
                    let timespan = timespan_input.timespan();

                    Action::Task(utils::failing_task(async move {
                        let transactions = finance_controller
                            .get_transactions_of_category(id, timespan)
                            .await?;
                        let accounts = finance_controller
                            .get_accounts_hash_map()
                            .await
                            .context("Error while fetching accounts")?;
                        let mut transaction_tuples = Vec::new();
                        for transaction in transactions {
                            let from_account = accounts
                                .get(&transaction.source)
                                .context(format!(
                                    "Could not find source account of transaction {}",
                                    transaction.source
                                ))?
                                .clone();
                            let to_account = accounts
                                .get(&transaction.destination)
                                .context(format!(
                                    "Could not find destination account of transaction {}",
                                    transaction.destination
                                ))?
                                .clone();
                            transaction_tuples.push((transaction, from_account, to_account));
                        }
                        let values = finance_controller
                            .get_relative_category_values(id, timespan)
                            .await?;
                        let categories = finance_controller.get_categories().await?;
                        let budgets = finance_controller.get_budgets().await?;
                        Ok(Message::Set(
                            cloned_category,
                            values,
                            transaction_tuples,
                            categories,
                            budgets,
                        ))
                    }))
                } else {
                    Action::None
                }
            }
            Message::Set(category, values, transactions, categories, budgets) => {
                let category_id = *category.id();
                *self = Self::Loaded {
                    category,
                    transaction_table: Box::new(utils::TransactionTable::new(
                        transactions,
                        categories,
                        budgets,
                        move |transaction| {
                            transaction
                                .categories
                                .get(&category_id)
                                .map(|sign| *sign == fm_core::Sign::Positive)
                        },
                    )),
                    values,
                    timespan_input: if let Self::Loaded { timespan_input, .. } = &self {
                        timespan_input.clone()
                    } else {
                        date_span_input::State::default()
                    },
                };
                Action::None
            }
            Message::TransactionTable(msg) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    match transaction_table.update(msg, finance_controller) {
                        utils::transaction_table::Action::None => Action::None,
                        utils::transaction_table::Action::ViewTransaction(id) => {
                            Action::ViewTransaction(id)
                        }
                        utils::transaction_table::Action::ViewAccount(id) => {
                            Action::ViewAccount(id)
                        }
                        utils::transaction_table::Action::Task(task) => {
                            Action::Task(task.map(Message::TransactionTable))
                        }
                    }
                } else {
                    Action::None
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if let Self::Loaded {
            category,
            transaction_table,
            values,
            timespan_input,
        } = self
        {
            super::view(
                "Category",
                utils::spaced_column![
                    utils::spaced_row![
                        utils::spaced_column![
                            utils::spal_row![
                                widget::text("Total value"),
                                widget::text(if let Some(value) = values.last() {
                                    value.1.to_string()
                                } else {
                                    "0€".to_string()
                                }),
                            ],
                            widget::text(category.name().to_string()),
                        ],
                        widget::Space::with_width(iced::Length::Fill),
                        utils::spaced_column![
                            utils::button::edit(Some(Message::Edit)),
                            utils::button::delete(Some(Message::Delete))
                        ]
                    ],
                    date_span_input::date_span_input(timespan_input)
                        .view()
                        .map(Message::ChangedTimespan),
                    transaction_table.view().map(Message::TransactionTable),
                ]
                .height(iced::Fill)
                .width(iced::Fill),
            )
        } else {
            widget::text("Loading...").into()
        }
    }
}
