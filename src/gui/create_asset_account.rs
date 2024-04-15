use crate::finance;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    NoteInput(String),
    IbanInput(String),
    BicInput(String),
    Submit,
}

#[derive(Debug, Clone)]
pub struct CreateAssetAccountDialog {
    name_input: String,
    note_input: String,
    iban_input: String,
    bic_input: String,
}

impl CreateAssetAccountDialog {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            note_input: String::new(),
            iban_input: String::new(),
            bic_input: String::new(),
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: &mut finance::FinanceManager,
    ) -> Option<finance::account::AssetAccount> {
        match message {
            Message::NameInput(input) => self.name_input = input,
            Message::NoteInput(input) => self.note_input = input,
            Message::IbanInput(input) => self.iban_input = input,
            Message::BicInput(input) => self.bic_input = input,
            Message::Submit => {
                let account = finance_manager.create_asset_account(
                    self.name_input.clone(),
                    if !self.note_input.is_empty() {
                        Some(self.note_input.clone())
                    } else {
                        None
                    },
                    Some(self.iban_input.clone()),
                    Some(self.bic_input.clone()),
                );
                return Some(account);
            }
        }
        None
    }

    pub fn view(&self) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
        iced::widget::column![
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
