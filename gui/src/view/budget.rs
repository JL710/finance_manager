use anyhow::{Context, Result};
use iced::widget;

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    Edit(fm_core::Id),
    Task(iced::Task<MessageContainer>),
    DeletedBudget,
}

#[derive(Debug, Clone)]
struct Init {
    budget: fm_core::Budget,
    value: fm_core::Currency,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    offset: i32,
    categories: Vec<fm_core::Category>,
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    Edit,
    IncreaseOffset,
    DecreaseOffset,
    Initialize(Box<Init>),
    TransactionTable(utils::transaction_table::Message),
    Delete,
    Deleted,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum View {
    NotLoaded,
    Loaded {
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transaction_table: utils::TransactionTable,
        offset: i32,
        time_span: fm_core::Timespan,
    },
}

impl View {
    pub fn new(
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        categories: Vec<fm_core::Category>,
        offset: i32,
    ) -> Result<Self> {
        let timespan = fm_core::calculate_budget_timespan(
            &budget,
            offset,
            fm_core::DateTime::now_utc().to_offset(fm_core::get_local_timezone()?),
        )?;
        Ok(Self::Loaded {
            budget: budget.clone(),
            current_value,
            transaction_table: utils::TransactionTable::new(
                transactions,
                categories,
                vec![budget],
                |transaction| Some(transaction.budget.unwrap().1 == fm_core::Sign::Positive),
            ),
            offset,
            time_span: timespan,
        })
    }

    pub fn fetch(
        id: fm_core::Id,
        offset: i32,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::NotLoaded,
            utils::failing_task(Self::initial_message(finance_controller, id, offset))
                .map(MessageContainer),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message.0 {
            Message::Initialize(init) => {
                match Self::new(
                    init.budget,
                    init.value,
                    init.transactions,
                    init.categories,
                    init.offset,
                ) {
                    Err(error) => {
                        return Action::Task(
                            iced::Task::future(utils::error_popup(utils::error_chain_string(
                                error,
                            )))
                            .discard(),
                        );
                    }
                    Ok(new) => *self = new,
                }
                Action::None
            }
            Message::Edit => {
                if let Self::Loaded { budget, .. } = self {
                    Action::Edit(*budget.id())
                } else {
                    Action::None
                }
            }
            Message::Delete => {
                if let Self::Loaded { budget, .. } = self {
                    if let rfd::MessageDialogResult::No = rfd::MessageDialog::new()
                        .set_title("Delete Budget")
                        .set_description(format!("Do you really want to delete {}?", budget.name()))
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .show()
                    {
                        return Action::None;
                    }

                    let id = *budget.id();
                    Action::Task(
                        utils::failing_task(async move {
                            finance_controller.delete_budget(id).await?;
                            Ok(Message::Deleted)
                        })
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
            }
            Message::Deleted => {
                *self = Self::NotLoaded;
                Action::DeletedBudget
            }
            Message::IncreaseOffset => {
                if let Self::Loaded { budget, offset, .. } = self {
                    Action::Task(
                        utils::failing_task(Self::initial_message(
                            finance_controller,
                            *budget.id(),
                            *offset + 1,
                        ))
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
            }
            Message::DecreaseOffset => {
                if let Self::Loaded { budget, offset, .. } = self {
                    Action::Task(
                        utils::failing_task(Self::initial_message(
                            finance_controller,
                            *budget.id(),
                            *offset - 1,
                        ))
                        .map(MessageContainer),
                    )
                } else {
                    Action::None
                }
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
            budget,
            current_value,
            transaction_table,
            offset,
            time_span,
        } = self
        {
            let mut column = utils::spaced_column![
                utils::spal_row![
                    widget::button("<").on_press(Message::DecreaseOffset),
                    widget::text!("Offset: {}", offset),
                    widget::text!(
                        "Time Span: {} - {}",
                        utils::date_time::to_date_time_string(time_span.0.unwrap()),
                        utils::date_time::to_date_time_string(time_span.1.unwrap())
                    ),
                    widget::button(">").on_press(Message::IncreaseOffset),
                ]
                .align_y(iced::Alignment::Center),
                widget::text!("Name: {}", budget.name()),
                widget::text!("Total Value: {}", budget.total_value()),
                widget::text!("Current Value: {}", current_value),
                widget::text!("Recurring: {}", budget.timespan())
            ];

            if let Some(content) = budget.description() {
                column = column.push(widget::text!("Description: {}", content));
            }

            super::view(
                "Budget",
                utils::spaced_column![
                    widget::row![
                        column,
                        widget::Space::with_width(iced::Length::Fill),
                        utils::spaced_row![
                            utils::button::edit(Some(Message::Edit)),
                            utils::button::delete(Some(Message::Delete))
                        ]
                    ],
                    widget::progress_bar(
                        0.0..=budget.total_value().get_eur_num() as f32,
                        current_value.get_eur_num() as f32
                    ),
                    transaction_table.view().map(Message::TransactionTable)
                ]
                .height(iced::Fill),
            )
            .map(MessageContainer)
        } else {
            widget::text("Loading...").into()
        }
    }

    async fn initial_message(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        id: fm_core::Id,
        offset: i32,
    ) -> Result<Message> {
        let budget = finance_controller
            .get_budget(id)
            .await?
            .context(format!("Could not find budget {}", id))?;
        let transactions = finance_controller
            .get_budget_transactions(
                &budget,
                offset,
                fm_core::get_local_timezone().context("Error while getting local timezone")?,
            )
            .await?;
        let current_value = finance_controller
            .get_budget_value(
                &budget,
                offset,
                fm_core::get_local_timezone().context("Error while getting local timezone")?,
            )
            .await?;

        let mut transaction_tuples = Vec::new();
        for transaction in transactions {
            let source = finance_controller
                .get_account(transaction.source)
                .await?
                .context(format!(
                    "Error while fetching account {}",
                    transaction.source
                ))?;
            let destination = finance_controller
                .get_account(transaction.destination)
                .await?
                .context(format!(
                    "Error while fetching account {}",
                    transaction.destination
                ))?;
            transaction_tuples.push((transaction, source, destination));
        }

        let categories = finance_controller.get_categories().await?;

        Ok(Message::Initialize(Box::new(Init {
            budget,
            value: current_value,
            transactions: transaction_tuples,
            offset,
            categories,
        })))
    }
}
