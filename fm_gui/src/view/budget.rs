use anyhow::Result;
use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    None,
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    Edit(fm_core::Id),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    Edit,
    IncreaseOffset,
    DecreaseOffset,
    Initialize(
        fm_core::Budget,
        fm_core::Currency,
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        i32,
        Vec<fm_core::Category>,
    ),
    TransactionTableMessage(utils::transaction_table::Message),
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Budget {
    NotLoaded,
    Loaded {
        budget: fm_core::Budget,
        current_value: fm_core::Currency,
        transaction_table: utils::TransactionTable,
        offset: i32,
        time_span: fm_core::Timespan,
    },
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
        categories: Vec<fm_core::Category>,
        offset: i32,
    ) -> Result<Self> {
        let timespan =
            fm_core::calculate_budget_timespan(&budget, offset, time::OffsetDateTime::now_utc())?;
        Ok(Self::Loaded {
            budget,
            current_value,
            transaction_table: utils::TransactionTable::new(
                transactions,
                categories,
                |transaction| Some(transaction.budget().unwrap().1 == fm_core::Sign::Positive),
            ),
            offset,
            time_span: timespan,
        })
    }

    pub fn fetch(
        id: fm_core::Id,
        offset: i32,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::NotLoaded,
            iced::Task::perform(Self::initial_message(finance_manager, id, offset), |x| {
                x.unwrap()
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize(budget, value, transactions, offset, categories) => {
                *self = Self::new(budget, value, transactions, categories, offset).unwrap();
                Action::None
            }
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::ViewTransaction(id) => Action::ViewTransaction(id),
            Message::Edit => {
                if let Self::Loaded { budget, .. } = self {
                    Action::Edit(*budget.id())
                } else {
                    Action::None
                }
            }
            Message::IncreaseOffset => {
                if let Self::Loaded { budget, offset, .. } = self {
                    Action::Task(iced::Task::perform(
                        Self::initial_message(finance_manager, *budget.id(), *offset + 1),
                        |x| x.unwrap(),
                    ))
                } else {
                    Action::None
                }
            }
            Message::DecreaseOffset => {
                if let Self::Loaded { budget, offset, .. } = self {
                    Action::Task(iced::Task::perform(
                        Self::initial_message(finance_manager, *budget.id(), *offset - 1),
                        |x| x.unwrap(),
                    ))
                } else {
                    Action::None
                }
            }
            Message::TransactionTableMessage(msg) => {
                if let Self::Loaded {
                    transaction_table, ..
                } = self
                {
                    match transaction_table.update(msg, finance_manager) {
                        utils::transaction_table::Action::None => Action::None,
                        utils::transaction_table::Action::ViewTransaction(id) => {
                            Action::ViewTransaction(id)
                        }
                        utils::transaction_table::Action::ViewAccount(id) => {
                            Action::ViewAccount(id)
                        }
                        utils::transaction_table::Action::Task(task) => {
                            Action::Task(task.map(Message::TransactionTableMessage))
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
            budget,
            current_value,
            transaction_table,
            offset,
            time_span,
        } = self
        {
            let mut column = widget::column![
                widget::row![
                    widget::button("<").on_press(Message::DecreaseOffset),
                    widget::text!("Offset: {}", offset),
                    widget::text!(
                        "Time Span: {} - {}",
                        time_span
                            .0
                            .unwrap()
                            .to_offset(fm_core::get_local_timezone().unwrap())
                            .format(
                                &time::format_description::parse("[day].[month].[year]").unwrap()
                            )
                            .unwrap(),
                        time_span
                            .1
                            .unwrap()
                            .to_offset(fm_core::get_local_timezone().unwrap())
                            .format(
                                &time::format_description::parse("[day].[month].[year]").unwrap()
                            )
                            .unwrap()
                    ),
                    widget::button(">").on_press(Message::IncreaseOffset),
                ]
                .align_y(iced::Alignment::Center)
                .spacing(10),
                widget::text!("Name: {}", budget.name()),
                widget::text!("Total Value: {}", budget.total_value()),
                widget::text!("Current Value: {}", current_value),
                widget::text!("Recurring: {}", budget.timespan())
            ]
            .spacing(10);

            if let Some(content) = budget.description() {
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
                    0.0..=budget.total_value().get_eur_num() as f32,
                    current_value.get_eur_num() as f32
                ),
                transaction_table
                    .view()
                    .map(Message::TransactionTableMessage)
            ]
            .height(iced::Fill)
            .spacing(10)
            .into()
        } else {
            widget::text("Loading...").into()
        }
    }

    async fn initial_message(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
        id: fm_core::Id,
        offset: i32,
    ) -> Result<Message> {
        let locked_manager = finance_manager.lock().await;
        let budget = locked_manager.get_budget(id).await?.unwrap();
        let transactions = locked_manager
            .get_budget_transactions(&budget, offset)?
            .await
            .unwrap();
        let current_value = locked_manager.get_budget_value(&budget, offset)?.await?;

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

        let categories = locked_manager.get_categories().await?;

        Ok(Message::Initialize(
            budget,
            current_value,
            transaction_tuples,
            offset,
            categories,
        ))
    }
}
