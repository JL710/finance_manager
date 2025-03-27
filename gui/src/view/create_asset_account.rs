use anyhow::Context;
use fm_core;
use iced::widget;

pub enum Action {
    None,
    AssetAccountCreated(fm_core::Id),
    Task(iced::Task<Message>),
    CancelWithId(fm_core::Id),
    Cancel,
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NoteInput(widget::text_editor::Action),
    IbanInput(String),
    BicInput(String),
    OffsetInput(components::currency_input::Action),
    Submit,
    AssetAccountCreated(fm_core::Id),
    Initialize(fm_core::account::AssetAccount),
    Cancel,
}

#[derive(Debug)]
pub struct View {
    id: Option<fm_core::Id>,
    name_input: String,
    note_input: widget::text_editor::Content,
    iban_input: String,
    bic_input: String,
    offset_input: components::currency_input::State,
    submitted: bool,
}

impl std::default::Default for View {
    fn default() -> Self {
        Self {
            offset_input: components::currency_input::State::new(fm_core::Currency::from(0.0)),
            id: None,
            name_input: String::new(),
            note_input: widget::text_editor::Content::default(),
            iban_input: String::new(),
            bic_input: String::new(),
            submitted: false,
        }
    }
}

impl View {
    pub fn fetch(
        account_id: fm_core::Id,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            components::failing_task(async move {
                let account = if let fm_core::account::Account::AssetAccount(acc) =
                    finance_controller
                        .get_account(account_id)
                        .await?
                        .context(format!("Could not find account {}", account_id))?
                {
                    acc
                } else {
                    anyhow::bail!("Error Account is not an asset account");
                };
                Ok(Message::Initialize(account))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Cancel => {
                if let Some(id) = self.id {
                    return Action::CancelWithId(id);
                } else {
                    return Action::Cancel;
                }
            }
            Message::Initialize(account) => {
                self.id = Some(account.id);
                self.name_input = account.name;
                self.note_input =
                    widget::text_editor::Content::with_text(&account.note.unwrap_or_default());
                self.iban_input = account.iban.map_or(String::new(), |iban| iban.to_string());
                self.bic_input = account.bic.map(|x| x.to_string()).unwrap_or_default();
                self.offset_input = components::currency_input::State::new(account.offset);
            }
            Message::AssetAccountCreated(id) => return Action::AssetAccountCreated(id),
            Message::NameInput(input) => self.name_input = input,
            Message::NoteInput(input) => self.note_input.perform(input),
            Message::IbanInput(input) => self.iban_input = input,
            Message::BicInput(input) => self.bic_input = input,
            Message::OffsetInput(action) => self.offset_input.perform(action),
            Message::Submit => {
                self.submitted = true;
                let name = self.name_input.clone();
                let note = if self.note_input.text().trim().is_empty() {
                    None
                } else {
                    Some(self.note_input.text())
                };
                let iban = if self.iban_input.is_empty() {
                    None
                } else {
                    Some(self.iban_input.parse().unwrap())
                };
                let bic = if self.bic_input.is_empty() {
                    None
                } else {
                    Some(self.bic_input.clone())
                };
                let offset = self.offset_input.currency().unwrap();
                let id = self.id;
                return Action::Task(components::failing_task(async move {
                    let account = if let Some(some_id) = id {
                        finance_controller
                            .update_asset_account(fm_core::account::AssetAccount::new(
                                some_id,
                                name,
                                note,
                                iban,
                                bic.map(|x| x.into()),
                                offset,
                            ))
                            .await?
                    } else {
                        finance_controller
                            .create_asset_account(name, note, iban, bic.map(|x| x.into()), offset)
                            .await?
                    };
                    Ok(Message::AssetAccountCreated(account.id))
                }));
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        if self.submitted {
            return "Loading...".into();
        }

        super::view(
            "Create Asset Account",
            widget::scrollable(components::spaced_column![
                components::labeled_entry("Name", &self.name_input, Message::NameInput, true),
                components::spaced_row![
                    "Notes",
                    widget::text_editor(&self.note_input).on_action(Message::NoteInput)
                ],
                components::labeled_entry("IBAN", &self.iban_input, Message::IbanInput, false),
                components::labeled_entry("BIC", &self.bic_input, Message::BicInput, false),
                components::spal_row![
                    "Offset",
                    components::currency_input::currency_input(&self.offset_input, true)
                        .view()
                        .map(Message::OffsetInput),
                ]
                .width(iced::Fill),
                components::submit_cancel_row(
                    if self.can_submit() {
                        Some(Message::Submit)
                    } else {
                        None
                    },
                    Some(Message::Cancel)
                ),
            ]),
        )
    }

    fn can_submit(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        if self.offset_input.currency().is_none() {
            return false;
        }
        true
    }
}
