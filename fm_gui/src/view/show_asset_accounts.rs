use fm_core::{self, FinanceManager};

use super::super::{utils, AppMessage, View};

use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            let accounts = finance_manager.lock().await.get_accounts().await;
            let mut tuples = Vec::new();
            for account in accounts {
                let amount = finance_manager
                    .lock()
                    .await
                    .get_account_sum(&account, chrono::Utc::now())
                    .await;
                tuples.push((account, amount));
            }
            tuples
        },
        |accounts| AppMessage::SwitchView(View::AssetAccounts(AssetAccountOverview::new(accounts))),
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateAssetAccount,
    AccountView(fm_core::account::AssetAccount),
}

#[derive(Debug, Clone)]
pub struct AssetAccountOverview {
    accounts: Vec<(fm_core::account::AssetAccount, fm_core::Currency)>,
}

impl AssetAccountOverview {
    pub fn new(accounts: Vec<(fm_core::account::Account, fm_core::Currency)>) -> Self {
        let asset_accounts = accounts
            .iter()
            .filter_map(|x| match &x.0 {
                fm_core::account::Account::AssetAccount(acc) => Some((acc.clone(), x.1.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();

        Self {
            accounts: asset_accounts,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::CreateAssetAccount => {
                return (
                    Some(View::CreateAssetAccountDialog(
                        super::create_asset_account::CreateAssetAccountDialog::new(),
                    )),
                    iced::Command::none(),
                );
            }
            Message::AccountView(account) => {
                return (
                    None,
                    iced::Command::perform(
                        async move {
                            super::view_account::ViewAccount::fetch(finance_manager, account.id())
                                .await
                        },
                        |view| AppMessage::SwitchView(View::ViewAccount(view)),
                    ),
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        let asset_accounts = self.accounts.clone();

        let account_list = iced::widget::Column::from_vec(
            asset_accounts
                .iter()
                .map(|x| asset_account_overview_entry(&x.0, &x.1))
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
    value: &fm_core::Currency,
) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
    widget::container(
        widget::button(
            widget::row![
                widget::text(account.name().to_owned()),
                widget::text(value.to_string())
            ]
            .spacing(30),
        )
        .on_press(Message::AccountView(account.clone())),
    )
    .style(utils::entry_row_container_style)
    .padding(10)
    .width(iced::Length::Fill)
    .into()
}
