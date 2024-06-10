use fm_core::{self, FinanceManager};

use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            let accounts = finance_manager
                .lock()
                .await
                .get_accounts()
                .await
                .unwrap()
                .iter()
                .filter_map(|x| match &x {
                    fm_core::account::Account::AssetAccount(acc) => Some(acc.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let mut tuples = Vec::new();
            for account in accounts {
                let amount = finance_manager
                    .lock()
                    .await
                    .get_account_sum(&account.clone().into(), chrono::Utc::now())
                    .await
                    .unwrap();
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
    pub fn new(accounts: Vec<(fm_core::account::AssetAccount, fm_core::Currency)>) -> Self {
        let asset_accounts = accounts;

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
            Message::CreateAssetAccount => (
                Some(View::CreateAssetAccountDialog(
                    super::create_asset_account::CreateAssetAccountDialog::new(),
                )),
                iced::Command::none(),
            ),
            Message::AccountView(account) => (
                None,
                iced::Command::perform(
                    async move {
                        super::view_account::ViewAccount::fetch(finance_manager, account.id()).await
                    },
                    |view| AppMessage::SwitchView(View::ViewAccount(view)),
                ),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let account_table =
            super::super::table_view::TableView::new(self.accounts.clone(), |account| {
                [
                    widget::button(widget::text(account.0.name().to_string()))
                        .on_press(Message::AccountView(account.0.clone()))
                        .padding(0)
                        .style(utils::button_link_style)
                        .into(),
                    utils::colored_currency_display(&account.1),
                ]
            })
            .headers(["Account".to_string(), "Current Value".to_string()]);

        widget::column![
            widget::row![widget::button("New AssetAccount").on_press(Message::CreateAssetAccount)],
            widget::horizontal_rule(10),
            account_table.into_element(),
        ]
        .into()
    }
}
