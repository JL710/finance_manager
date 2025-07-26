use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    Task(iced::Task<Message>),
    None,
    FinishedImport,
}

#[derive(Debug, Clone)]
pub enum Message {
    Action(Box<fm_importer::action::Action>),
    Next,
    Finished,
    SelectOption(Option<usize>),
}

#[derive(Debug)]
pub struct Import<
    P: fm_importer::Parser + fm_core::MaybeSend + std::marker::Sync,
    FM: fm_core::FinanceManager + 'static,
> {
    importer: Arc<Mutex<fm_importer::Importer<FM, P>>>,
    action: Option<(fm_importer::action::Action, Option<usize>)>,
}

impl<
    P: fm_importer::Parser + fm_core::MaybeSend + std::marker::Sync + 'static,
    FM: fm_core::FinanceManager,
> Import<P, FM>
{
    pub fn new(importer: Arc<Mutex<fm_importer::Importer<FM, P>>>) -> (Self, iced::Task<Message>) {
        (
            Self {
                importer,
                action: None,
            },
            iced::Task::done(Message::Next),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Action(action) => {
                self.action = Some((*action, None));
                Action::None
            }
            Message::Next => {
                if let Some((action, selected_option)) = &self.action {
                    let mut action = action.clone();
                    match &mut action {
                        fm_importer::action::Action::DestinationAccountExists(obj_exists) => {
                            obj_exists.select_object(
                                selected_option.map(|x| obj_exists.possible_objects()[x].clone()),
                            )
                        }
                        fm_importer::action::Action::SourceAccountExists(obj_exists) => obj_exists
                            .select_object(
                                selected_option.map(|x| obj_exists.possible_objects()[x].clone()),
                            ),
                        fm_importer::action::Action::TransactionExists(obj_exists) => obj_exists
                            .select_object(
                                selected_option.map(|x| obj_exists.possible_objects()[x].clone()),
                            ),
                        _ => {}
                    }
                    self.action = None;
                    let importer = self.importer.clone();
                    Action::Task(error::failing_task(async move {
                        let mut importer = importer.lock().await;
                        // perform actions as long as needed
                        if !matches!(action, fm_importer::action::Action::None) {
                            action = importer.perform(action).await?;
                            if !matches!(action, fm_importer::action::Action::None) {
                                return Ok(Message::Action(Box::new(action)));
                            }
                        }
                        // new entries
                        while matches!(action, fm_importer::action::Action::None) {
                            if let Some(new_action) = importer.next().await? {
                                action = new_action;
                            } else {
                                return Ok(Message::Finished);
                            }
                        }
                        Ok(Message::Action(Box::new(action)))
                    }))
                } else {
                    let importer = self.importer.clone();
                    Action::Task(error::failing_task(async move {
                        let mut importer = importer.lock().await;
                        let mut action = fm_importer::action::Action::None;
                        while matches!(action, fm_importer::action::Action::None) {
                            if let Some(new_action) = importer.next().await? {
                                action = new_action;
                            } else {
                                return Ok(Message::Finished);
                            }
                        }
                        Ok(Message::Action(Box::new(action)))
                    }))
                }
            }
            Message::SelectOption(option) => {
                if let Some((_, selected_option)) = &mut self.action {
                    *selected_option = option;
                }
                Action::None
            }
            Message::Finished => Action::FinishedImport,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        components::spal_column![
            widget::container(widget::scrollable(
                widget::container(
                    widget::container(if let Some((action, selected_option)) = &self.action {
                        match action {
                            fm_importer::action::Action::None => {
                                iced::Element::new(widget::text(""))
                            }
                            fm_importer::action::Action::DestinationAccountExists(
                                account_exists,
                            ) => obj_exists(
                                account_exists,
                                *selected_option,
                                account_display,
                                Message::SelectOption,
                            ),
                            fm_importer::action::Action::SourceAccountExists(account_exists) => {
                                obj_exists(
                                    account_exists,
                                    *selected_option,
                                    account_display,
                                    Message::SelectOption,
                                )
                            }
                            fm_importer::action::Action::TransactionExists(transaction_exists) => {
                                obj_exists(
                                    transaction_exists,
                                    *selected_option,
                                    transaction_display,
                                    Message::SelectOption,
                                )
                            }
                        }
                    } else {
                        iced::Element::new(widget::text(""))
                    })
                    .style(widget::container::rounded_box)
                    .padding(style::PADDING)
                )
                .center_x(iced::Fill),
            ))
            .center_y(iced::Fill),
            widget::row![
                widget::horizontal_space(),
                widget::button("Next").on_press_maybe(if self.action.is_some() {
                    Some(Message::Next)
                } else {
                    None
                })
            ]
        ]
        .into()
    }
}

fn account_entry_display<'a, Message: 'a>(
    account: &'a Option<fm_core::account::Account>,
    account_entry: &'a fm_importer::AccountEntry,
) -> iced::Element<'a, Message> {
    if let Some(account) = account {
        account_display(account)
    } else {
        let mut col = components::spaced_column![];
        if let Some(name) = account_entry.name() {
            col = col.push(components::spaced_row!["Name: ", widget::text(name)])
        }
        col = col.push(components::spaced_row![
            "Account ID: ",
            widget::text(account_entry.iban().to_string())
        ]);
        if let Some(bic) = account_entry.bic() {
            col = col.push(components::spaced_row![
                "Bic: ",
                widget::text(bic.to_string())
            ]);
        }
        col.into()
    }
}

fn transaction_entry_display<'a, Message: 'a>(
    entry: &'a fm_importer::TransactionEntry,
) -> iced::Element<'a, Message> {
    components::spaced_column![
        components::spaced_row!["Title: ", widget::text(&entry.title)],
        components::spaced_row!["Description: ", widget::text(&entry.description)],
        components::spaced_row!["Value: ", widget::text(entry.value.to_string())],
        components::spaced_row![
            "Date: ",
            widget::text(components::date_time::to_date_time_string(
                time::PrimitiveDateTime::new(entry.date.date(), entry.date.time())
            ))
        ],
        components::spaced_row![
            "Source Account: ",
            account_entry_display(&entry.source_account, &entry.source_entry)
        ],
        components::spaced_row![
            "Destination Account: ",
            account_entry_display(&entry.destination_account, &entry.destination_entry)
        ]
    ]
    .into()
}

fn obj_exists<'a, T: Clone, Message: Clone + 'a>(
    obj_exists: &'a fm_importer::action::ObjectExists<T>,
    selected_option: Option<usize>,
    display_t: impl Fn(&'a T) -> iced::Element<'a, Message>,
    select_message: impl Fn(Option<usize>) -> Message + Clone,
) -> iced::Element<'a, Message> {
    let mut option_column = components::LineSeparatedColumn::default().push(components::spal_row![
        widget::radio("", None, Some(selected_option), select_message.clone()),
        "New / Does not exist"
    ]);
    for (i, t) in obj_exists.possible_objects().iter().enumerate() {
        option_column = option_column.push(components::spal_row![
            widget::radio("", Some(i), Some(selected_option), select_message.clone()),
            display_t(t)
        ])
    }

    widget::container(
        components::LineSeparatedColumn::default()
            .spacing(style::SPACING)
            .align_x(iced::Alignment::Center)
            .push(transaction_entry_display(obj_exists.transaction_entry()))
            .push(option_column),
    )
    .into()
}

fn account_display<'a, Message: 'a>(
    account: &'a fm_core::account::Account,
) -> iced::Element<'a, Message> {
    let mut col = components::spaced_column!();
    match account {
        fm_core::account::Account::AssetAccount(_) => col = col.push("Asset Account"),
        fm_core::account::Account::BookCheckingAccount(_) => {
            col = col.push("Book Checking Account")
        }
    }
    col = col.push(components::spaced_row!["Name: ", account.name()]);
    if let Some(iban) = account.iban() {
        col = col.push(components::spaced_row![
            "IBAN/Account ID: ",
            widget::text(iban.to_string())
        ]);
    }
    if let Some(bic) = account.bic() {
        col = col.push(components::spaced_row![
            "BIC: ",
            widget::text(bic.to_string())
        ]);
    }
    if let Some(note) = account.note() {
        col = col.push(components::spaced_row!["Note: ", widget::text(note)]);
    }
    if let fm_core::account::Account::AssetAccount(asset_account) = account {
        col = col.push(components::spaced_row![
            "Offset: ",
            widget::text(asset_account.offset.to_string())
        ]);
    }

    col.into()
}

fn transaction_display(transaction: &fm_core::Transaction) -> iced::Element<'_, Message> {
    let mut col = components::spaced_column![widget::text!("Title: {}", transaction.title),];
    if let Some(description) = &transaction.description {
        col = col.push(components::spaced_row![
            "Description: ",
            widget::text(description)
        ]);
    }
    col = col.push(components::spaced_row![
        "Amount: ",
        widget::text(transaction.amount().to_string())
    ]);
    col.into()
}
