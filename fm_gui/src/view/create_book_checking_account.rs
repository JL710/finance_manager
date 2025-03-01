use fm_core;

use iced::widget;

use anyhow::Context;
use async_std::sync::Mutex;
use std::sync::Arc;

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
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
        account_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            View::default(),
            utils::failing_task(async move {
                let account = finance_manager
                    .lock()
                    .await
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
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::AccountCreated(id) => return Action::AccountCreated(id),
            Message::Initialize(account) => {
                self.id = Some(account.id());
                self.name_input = account.name().to_string();
                self.note_input = account
                    .note()
                    .map_or(widget::text_editor::Content::default(), |note| {
                        widget::text_editor::Content::with_text(note)
                    });
                self.iban_input = account
                    .iban()
                    .clone()
                    .map_or(String::new(), |iban| iban.to_string());
                self.bic_input = account.bic().map_or(String::new(), |bic| bic.to_string());
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
                return Action::Task(utils::failing_task(async move {
                    let account = if let Some(some_id) = id {
                        finance_manager
                            .lock()
                            .await
                            .update_book_checking_account(some_id, name, note, iban, bic)
                            .await?
                    } else {
                        finance_manager
                            .lock()
                            .await
                            .create_book_checking_account(name, note, iban, bic)
                            .await?
                    };
                    Ok(Message::AccountCreated(account.id()))
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

        super::view(
            "Create Book Checking Account",
            widget::scrollable(utils::spaced_column![
                utils::labeled_entry("Name", &self.name_input, Message::NameInput, true),
                utils::spaced_row![
                    widget::text("Notes"),
                    widget::text_editor(&self.note_input).on_action(Message::NoteInput)
                ],
                utils::labeled_entry("IBAN", &self.iban_input, Message::IbanInput, false),
                utils::labeled_entry("BIC", &self.bic_input, Message::BicInput, false),
                utils::submit_cancel_row(
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
        true
    }
}
