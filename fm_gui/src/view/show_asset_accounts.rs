use fm_core::{self, FinanceManager};

use super::super::{AppMessage, View};

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
        let mut account_table = super::super::table::Table::new(2)
            .set_headers(vec!["Name".to_owned(), "Current Value".to_owned()]);

        for account in &self.accounts {
            account_table.push_row(vec![
                widget::button(account.0.name())
                    .style(|theme: &iced::Theme, status| widget::button::Style {
                        background: None,
                        text_color: theme.palette().text,
                        ..Default::default()
                    })
                    .padding(0)
                    .on_press(Message::AccountView(account.0.clone()))
                    .into(),
                widget::Text::new(account.1.to_string()).into(),
            ]);
        }

        widget::column![
            widget::row![widget::button("New AssetAccount").on_press(Message::CreateAssetAccount)],
            widget::horizontal_rule(10),
            account_table.convert_to_view(),
        ]
        .into()
    }
}
