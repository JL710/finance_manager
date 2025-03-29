use fm_core;

use iced::widget;

use anyhow::Context;

pub enum Action {
    None,
    AccountCreated(fm_core::Id),
    Task(iced::Task<Message>),
    Cancel,
    CancelWithId(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NoteInput(widget::text_editor::Action),
    IbanInput(String),
    BicInput(String),
    Submit,
    Initialize(fm_core::account::BookCheckingAccount),
    AccountCreated(fm_core::Id),
    Cancel,
}

#[derive(Debug, Default)]
pub struct View {
    id: Option<fm_core::Id>,
    name_input: String,
    note_input: widget::text_editor::Content,
    iban_input: String,
    bic_input: String,
    submitted: bool,
}

impl View {
    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
        account_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            View::default(),
            error::failing_task(async move {
                let account = finance_controller
                    .get_account(account_id)
                    .await?
                    .context(format!("Could not find account {}", account_id))?;
                if let fm_core::account::Account::BookCheckingAccount(acc) = account {
                    Ok(Message::Initialize(acc))
                } else {
                    anyhow::bail!("Wrong account type");
                }
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::AccountCreated(id) => return Action::AccountCreated(id),
            Message::Initialize(account) => {
                self.id = Some(account.id);
                self.name_input = account.name;
                self.note_input =
                    widget::text_editor::Content::with_text(&account.note.unwrap_or_default());
                self.iban_input = account.iban.map_or(String::new(), |iban| iban.to_string());
                self.bic_input = account.bic.map_or(String::new(), |bic| bic.to_string());
            }
            Message::NameInput(input) => self.name_input = input,
            Message::NoteInput(action) => self.note_input.perform(action),
            Message::IbanInput(input) => self.iban_input = input,
            Message::BicInput(input) => self.bic_input = input,
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
                let id = self.id;
                return Action::Task(error::failing_task(async move {
                    let account = if let Some(some_id) = id {
                        finance_controller
                            .update_book_checking_account(
                                fm_core::account::BookCheckingAccount::new(
                                    some_id,
                                    name,
                                    note,
                                    iban,
                                    bic.map(|x| x.into()),
                                ),
                            )
                            .await?
                    } else {
                        finance_controller
                            .create_book_checking_account(name, note, iban, bic.map(|x| x.into()))
                            .await?
                    };
                    Ok(Message::AccountCreated(account.id))
                }));
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

    pub fn view(&self) -> iced::Element<Message> {
        if self.submitted {
            return "Loading...".into();
        }

        widget::scrollable(components::spaced_column![
            components::labeled_entry("Name", &self.name_input, Message::NameInput, true),
            components::spaced_row![
                "Notes",
                widget::text_editor(&self.note_input).on_action(Message::NoteInput)
            ],
            components::labeled_entry("IBAN", &self.iban_input, Message::IbanInput, false),
            components::labeled_entry("BIC", &self.bic_input, Message::BicInput, false),
            components::submit_cancel_row(
                if self.can_submit() {
                    Some(Message::Submit)
                } else {
                    None
                },
                Some(Message::Cancel)
            ),
        ])
        .into()
    }

    fn can_submit(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        true
    }
}
