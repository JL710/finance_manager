use anyhow::{Context, Result};
use components::date_time::date_span_input;
use iced::widget;

pub enum Action {
    None,
    EditCategory(fm_core::Id),
    DeleteCategory(iced::Task<()>),
    Task(iced::Task<MessageContainer>),
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
struct Init {
    category: fm_core::Category,
    sums: Vec<(fm_core::DateTime, fm_core::Currency)>,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    categories: Vec<fm_core::Category>,
    budgets: Vec<fm_core::Budget>,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    Delete,
    Edit,
    ChangedTimespan(date_span_input::Action),
    Set(Init),
    Reload(Option<Init>),
    TransactionTable(components::transaction_table::Message),
}

#[derive(Debug)]
pub enum View {
    NotLoaded,
    Loaded {
        category: fm_core::Category,
        transaction_table: Box<components::TransactionTable>,
        values: Vec<(fm_core::DateTime, fm_core::Currency)>,
        timespan_input: date_span_input::State,
    },
}

impl View {
    pub fn reload(
        &mut self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<MessageContainer> {
        let mut task = iced::Task::none();
        if let Self::Loaded {
            category,
            timespan_input,
            ..
        } = self
        {
            task = error::failing_task(Self::init_future(
                finance_controller,
                category.id,
                components::date_time::date_span_to_time_span(
                    timespan_input.timespan(),
                    utc_offset,
                ),
            ))
            .map(Message::Reload)
            .map(MessageContainer);
            *self = Self::NotLoaded;
        }
        task
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        category_id: fm_core::Id,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::NotLoaded,
            error::failing_task(async move {
                Self::init_future(finance_controller, category_id, (None, None))
                    .await?
                    .context("Could not find category")
            })
            .map(Message::Set)
            .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        utc_offset: time::UtcOffset,
    ) -> Action {
        let message = message.0;
        match message {
            Message::Reload(init) => {
                if let Some(init) = init {
                    if let Self::Loaded {
                        category,
                        transaction_table,
                        values,
                        ..
                    } = self
                    {
                        *category = init.category;
                        *values = init.sums;
                        transaction_table.reload(init.transactions, init.categories, init.budgets);
                    }
                } else {
                    *self = Self::NotLoaded;
                }
                Action::None
            }
            Message::Delete => {
                if let Self::Loaded { category, .. } = self {
                    if let rfd::MessageDialogResult::No = rfd::MessageDialog::new()
                        .set_title("Delete category")
                        .set_description(format!("Do you really want to delete {}?", category.name))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        return Action::None;
                    }

                    let category_id = category.id;
                    Action::DeleteCategory(error::failing_task(async move {
                        finance_controller.delete_category(category_id).await?;
                        Ok(())
                    }))
                } else {
                    Action::None
                }
            }
            Message::Edit => {
                if let Self::Loaded { category, .. } = self {
                    Action::EditCategory(category.id)
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

                    let id: u64 = category.id;
                    let timespan = timespan_input.timespan();

                    Action::Task(
                        error::failing_task(async move {
                            Self::init_future(
                                finance_controller,
                                id,
                                components::date_time::date_span_to_time_span(timespan, utc_offset),
                            )
                            .await?
                            .context("category not found")
                        })
                        .map(Message::Set)
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
            }
            Message::Set(init) => {
                let category_id = init.category.id;
                *self = Self::Loaded {
                    category: init.category,
                    transaction_table: Box::new(components::TransactionTable::new(
                        init.transactions,
                        init.categories,
                        init.budgets,
                        move |transaction| {
                            transaction
                                .categories
                                .get(&category_id)
                                .map(|sign| *sign == fm_core::Sign::Positive)
                        },
                    )),
                    values: init.sums,
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
                        components::transaction_table::Action::None => Action::None,
                        components::transaction_table::Action::ViewTransaction(id) => {
                            Action::ViewTransaction(id)
                        }
                        components::transaction_table::Action::ViewAccount(id) => {
                            Action::ViewAccount(id)
                        }
                        components::transaction_table::Action::Task(task) => {
                            Action::Task(task.map(Message::TransactionTable).map(MessageContainer))
                        }
                    }
                } else {
                    Action::None
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        if let Self::Loaded {
            category,
            transaction_table,
            values,
            timespan_input,
        } = self
        {
            iced::Element::new(
                components::spaced_column![
                    components::spaced_row![
                        components::spaced_column![
                            components::spal_row![
                                "Total value",
                                widget::text(if let Some(value) = values.last() {
                                    value.1.to_string()
                                } else {
                                    "0€".to_string()
                                }),
                            ],
                            category.name.as_str(),
                        ],
                        widget::Space::with_width(iced::Length::Fill),
                        components::spaced_column![
                            components::button::edit(Some(Message::Edit)),
                            components::button::delete(Some(Message::Delete))
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
            .map(MessageContainer)
        } else {
            widget::text("Loading...").into()
        }
    }

    async fn init_future(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        category_id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Option<Init>> {
        let category = if let Some(category) = finance_controller.get_category(category_id).await? {
            category
        } else {
            return Ok(None);
        };
        let transactions = finance_controller
            .get_transactions_of_category(category_id, timespan)
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
            .get_relative_category_values(category_id, timespan)
            .await?;

        let categories = finance_controller.get_categories().await?;
        let budgets = finance_controller.get_budgets().await?;
        Ok(Some(Init {
            category,
            sums: values,
            transactions: transaction_tuples,
            categories,
            budgets,
        }))
    }
}
