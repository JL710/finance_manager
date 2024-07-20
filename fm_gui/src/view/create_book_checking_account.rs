use super::super::utils;
use fm_core;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    CreateAccount(iced::Task<fm_core::Id>),
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NoteInput(String),
    IbanInput(String),
    BicInput(String),
    Submit,
    Initialize(fm_core::account::BookCheckingAccount),
}

#[derive(Debug, Clone, Default)]
pub struct CreateBookCheckingAccount {
    id: Option<fm_core::Id>,
    name_input: String,
    note_input: String,
    iban_input: String,
    bic_input: String,
}

impl CreateBookCheckingAccount {
    pub fn new(account: fm_core::account::BookCheckingAccount) -> Self {
        Self {
            id: Some(account.id()),
            name_input: account.name().to_string(),
            note_input: account
                .note()
                .map_or(String::new(), |note| note.to_string()),
            iban_input: account
                .iban()
                .map_or(String::new(), |iban| iban.to_string()),
            bic_input: account.bic().map_or(String::new(), |bic| bic.to_string()),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
        account_id: fm_core::Id,
    ) -> (Self, iced::Task<Message>) {
        (
            CreateBookCheckingAccount::default(),
            iced::Task::future(async move {
                let account = finance_manager
                    .lock()
                    .await
                    .get_account(account_id)
                    .await
                    .unwrap()
                    .unwrap();
                if let fm_core::account::Account::BookCheckingAccount(acc) = account {
                    Message::Initialize(acc)
                } else {
                    panic!("Wrong account type")
                }
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::Initialize(account) => {
                self.id = Some(account.id());
                self.name_input = account.name().to_string();
                self.note_input = account
                    .note()
                    .map_or(String::new(), |note| note.to_string());
                self.iban_input = account
                    .iban()
                    .map_or(String::new(), |iban| iban.to_string());
                self.bic_input = account.bic().map_or(String::new(), |bic| bic.to_string());
            }
            Message::NameInput(input) => self.name_input = input,
            Message::NoteInput(input) => self.note_input = input,
            Message::IbanInput(input) => self.iban_input = input,
            Message::BicInput(input) => self.bic_input = input,
            Message::Submit => {
                let name = self.name_input.clone();
                let note = if self.note_input.is_empty() {
                    None
                } else {
                    Some(self.note_input.clone())
                };
                let iban = if self.iban_input.is_empty() {
                    None
                } else {
                    Some(self.iban_input.clone())
                };
                let bic = if self.bic_input.is_empty() {
                    None
                } else {
                    Some(self.bic_input.clone())
                };
                let id = self.id;
                return Action::CreateAccount(iced::Task::future(async move {
                    let account = if let Some(some_id) = id {
                        finance_manager
                            .lock()
                            .await
                            .update_book_checking_account(some_id, name, note, iban, bic)
                            .await
                            .unwrap()
                    } else {
                        finance_manager
                            .lock()
                            .await
                            .create_book_checking_account(name, note, iban, bic)
                            .await
                            .unwrap()
                    };
                    account.id()
                }));
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'static, Message> {
        iced::widget::column![
            utils::heading("Create Book Checking Account", utils::HeadingLevel::H1),
            iced::widget::row![
                iced::widget::text("Name"),
                iced::widget::text_input("Name", &self.name_input).on_input(Message::NameInput)
            ]
            .spacing(10),
            iced::widget::row![
                iced::widget::text("Notes"),
                iced::widget::text_input("Notes", &self.note_input).on_input(Message::NoteInput)
            ]
            .spacing(10),
            iced::widget::row![
                iced::widget::text("IBAN"),
                iced::widget::text_input("IBAN", &self.iban_input).on_input(Message::IbanInput)
            ]
            .spacing(10),
            iced::widget::row![
                iced::widget::text("BIC"),
                iced::widget::text_input("BIC", &self.bic_input).on_input(Message::BicInput)
            ]
            .spacing(10),
            iced::widget::button("Submit").on_press_maybe(if self.can_submit() {
                Some(Message::Submit)
            } else {
                None
            })
        ]
        .spacing(10)
        .into()
    }

    fn can_submit(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        true
    }
}
