use super::super::utils;
use fm_core;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    CreateAssetAccount(iced::Task<fm_core::Id>),
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NoteInput(String),
    IbanInput(String),
    BicInput(String),
    OffsetInput(String),
    Submit,
    Initialize(fm_core::account::AssetAccount),
}

#[derive(Debug, Clone, Default)]
pub struct CreateAssetAccountDialog {
    id: Option<fm_core::Id>,
    name_input: String,
    note_input: String,
    iban_input: String,
    bic_input: String,
    offset_input: String,
}

impl CreateAssetAccountDialog {
    pub fn fetch(
        account_id: fm_core::Id,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            iced::Task::future(async move {
                let account = if let fm_core::account::Account::AssetAccount(acc) = finance_manager
                    .lock()
                    .await
                    .get_account(account_id)
                    .await
                    .unwrap()
                    .unwrap()
                {
                    acc
                } else {
                    panic!("Account is not an asset account")
                };
                Message::Initialize(account)
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
                self.note_input = account.note().unwrap_or_default().to_string();
                self.iban_input = account.iban().unwrap_or_default().to_string();
                self.bic_input = account.bic().unwrap_or_default().to_string();
                self.offset_input = account.offset().to_string();
            }
            Message::NameInput(input) => self.name_input = input,
            Message::NoteInput(input) => self.note_input = input,
            Message::IbanInput(input) => self.iban_input = input,
            Message::BicInput(input) => self.bic_input = input,
            Message::OffsetInput(input) => self.offset_input = input,
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
                let offset = fm_core::Currency::Eur(self.offset_input.parse().unwrap());
                let id = self.id;
                return Action::CreateAssetAccount(iced::Task::future(async move {
                    let account = if let Some(some_id) = id {
                        finance_manager
                            .lock()
                            .await
                            .update_asset_account(some_id, name, note, iban, bic, offset)
                            .await
                            .unwrap()
                    } else {
                        finance_manager
                            .lock()
                            .await
                            .create_asset_account(name, note, iban, bic, offset)
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
            utils::heading("Create Asset Account", utils::HeadingLevel::H1),
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
            iced::widget::row![
                iced::widget::text("Offset"),
                iced::widget::text_input("Offset", &self.offset_input)
                    .on_input(Message::OffsetInput)
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
        if self.offset_input.parse::<f64>().is_err() {
            return false;
        }
        true
    }
}
