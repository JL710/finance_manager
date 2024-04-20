use fm_core;

use super::super::{utils, AppMessage, View};

use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move { finance_manager.lock().await.get_accounts().await },
        |accounts| AppMessage::SwitchView(View::AssetAccounts(AssetAccountOverview::new(accounts))),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateAssetAccount,
}

#[derive(Debug, Clone)]
pub struct AssetAccountOverview {
    accounts: Vec<fm_core::account::AssetAccount>,
}

impl AssetAccountOverview {
    pub fn new(accounts: Vec<fm_core::account::Account>) -> Self {
        let asset_accounts = accounts
            .iter()
            .filter_map(|x| match x {
                fm_core::account::Account::AssetAccount(acc) => Some(acc.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        Self {
            accounts: asset_accounts,
        }
    }

    pub fn update(&mut self, message: Message) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::CreateAssetAccount => {
                return (
                    Some(View::CreateAssetAccountDialog(
                        super::create_asset_account::CreateAssetAccountDialog::new(),
                    )),
                    iced::Command::none(),
                );
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        let asset_accounts = self.accounts.clone();

        let account_list = iced::widget::Column::from_vec(
            asset_accounts
                .iter()
                .map(asset_account_overview_entry)
                .collect(),
        )
        .width(iced::Length::Fill);

        iced::widget::column![
            iced::widget::row![
                iced::widget::button("New AssetAccount").on_press(Message::CreateAssetAccount)
            ],
            iced::widget::horizontal_rule(10),
            iced::widget::scrollable(account_list)
        ]
        .into()
    }
}

fn asset_account_overview_entry(
    account: &fm_core::account::AssetAccount,
) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
    iced::widget::container(iced::widget::text(account.name().to_owned()))
        .style(utils::entry_row_container_style)
        .padding(10)
        .width(iced::Length::Fill)
        .into()
}
