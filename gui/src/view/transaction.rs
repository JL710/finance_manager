use anyhow::Context;
use iced::widget;

#[derive(Debug, Clone)]
struct Init {
    transaction: fm_core::Transaction,
    source: fm_core::account::Account,
    destination: fm_core::account::Account,
    budget: Option<fm_core::Budget>,
    categories: Vec<fm_core::Category>,
}

#[allow(clippy::large_enum_variant)]
pub enum Action {
    None,
    Edit(fm_core::Id),
    Delete(iced::Task<()>),
    ViewAccount(fm_core::Id),
    ViewBudget(fm_core::Id),
    ViewCategory(fm_core::Id),
    NewBillWithTransaction(fm_core::Transaction),
}

#[derive(Debug, Clone)]
pub struct MessageContainer(Message);

#[derive(Debug, Clone)]
enum Message {
    Edit,
    Delete,
    ViewAccount(fm_core::Id),
    ViewBudget(fm_core::Id),
    ViewCategory(fm_core::Id),
    Initialize(Box<Init>),
    NewBill,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum View {
    NotLoaded,
    Loaded {
        transaction: fm_core::Transaction,
        source: fm_core::account::Account,
        destination: fm_core::account::Account,
        budget: Option<fm_core::Budget>,
        categories: Vec<fm_core::Category>,
    },
}

impl View {
    pub fn fetch(
        id: fm_core::Id,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<MessageContainer>) {
        (
            Self::NotLoaded,
            error::failing_task(async move {
                let transaction = finance_controller
                    .get_transaction(id)
                    .await?
                    .context(format!("Could not find transaction {}", id))?;
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
                    Some(budget_id) => finance_controller.get_budget(budget_id.0).await?,
                    None => None,
                };
                let categories = finance_controller.get_categories().await?;
                Ok(MessageContainer(Message::Initialize(Box::new(Init {
                    transaction,
                    source,
                    destination,
                    budget,
                    categories,
                }))))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: MessageContainer,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message.0 {
            Message::Initialize(init) => {
                *self = Self::Loaded {
                    transaction: init.transaction,
                    source: init.source,
                    destination: init.destination,
                    budget: init.budget,
                    categories: init.categories,
                };
                Action::None
            }
            Message::NewBill => {
                if let Self::Loaded { transaction, .. } = self {
                    Action::NewBillWithTransaction(transaction.clone())
                } else {
                    Action::None
                }
            }
            Message::Edit => {
                if let Self::Loaded { transaction, .. } = self {
                    Action::Edit(transaction.id)
                } else {
                    Action::None
                }
            }
            Message::Delete => {
                match rfd::MessageDialog::new()
                    .set_title("Delete Transaction")
                    .set_description("Are you sure you want to delete this transaction?")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()
                {
                    rfd::MessageDialogResult::Yes => (),
                    _ => return Action::None,
                }
                if let Self::Loaded { transaction, .. } = self {
                    let id = transaction.id;
                    Action::Delete(error::failing_task(async move {
                        finance_controller.delete_transaction(id).await?;
                        Ok(())
                    }))
                } else {
                    Action::None
                }
            }
            Message::ViewAccount(acc) => Action::ViewAccount(acc),
            Message::ViewBudget(budget) => Action::ViewBudget(budget),
            Message::ViewCategory(category) => Action::ViewCategory(category),
        }
    }

    pub fn view(&self) -> iced::Element<'_, MessageContainer> {
        if let Self::Loaded {
            transaction,
            source,
            destination,
            budget,
            categories,
        } = self
        {
            let mut column = widget::column![
                widget::row![widget::text!("Value: {}", transaction.amount())],
                widget::text!("Name: {}", transaction.title),
                components::link(widget::text!("Source: {}", source))
                    .on_press(Message::ViewAccount(*source.id())),
                components::link(widget::text!("Destination: {}", destination))
                    .on_press(Message::ViewAccount(*destination.id())),
                widget::text!(
                    "Date: {}",
                    components::date_time::to_date_time_string(
                        components::date_time::offset_to_primitive(transaction.date)
                    )
                ),
            ];

            if let Some(budget) = &budget {
                column = column.push(components::spal_row![
                    components::link(widget::text!("Budget: {}", &budget.name))
                        .on_press(Message::ViewBudget(budget.id)),
                    widget::checkbox(
                        "Negative",
                        transaction
                            .budget
                            .is_some_and(|x| x.1 == fm_core::Sign::Negative)
                    )
                ]);
            }

            if let Some(content) = &transaction.description {
                column = column.push(components::spaced_row![
                    "Description: ",
                    widget::container(content.as_str())
                        .padding(3)
                        .style(style::container_style_background_weak)
                ]);
            }

            let mut category_column = components::spaced_column!();
            for category in &transaction.categories {
                category_column = category_column.push(components::spal_row![
                    widget::checkbox("", true,),
                    widget::button(
                        categories
                            .iter()
                            .find(|x| x.id == *category.0)
                            .unwrap()
                            .name
                            .as_str()
                    )
                    .style(widget::button::text)
                    .on_press(Message::ViewCategory(*category.0)),
                    widget::checkbox("Negative", *category.1 == fm_core::Sign::Negative)
                ]);
            }

            iced::Element::new(widget::column![
                widget::row![
                    column,
                    widget::Space::with_width(iced::Length::Fill),
                    components::spaced_column![
                        components::button::edit(Some(Message::Edit)),
                        components::button::delete(Some(Message::Delete)),
                        components::button::new("New Bill", Some(Message::NewBill))
                    ]
                ],
                widget::horizontal_rule(10),
                widget::scrollable(category_column)
            ])
            .map(MessageContainer)
        } else {
            widget::text("Loading...").into()
        }
    }
}
