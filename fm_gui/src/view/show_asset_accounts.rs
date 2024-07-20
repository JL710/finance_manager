use fm_core::{self, FinanceManager};

use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    iced::Task::perform(
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
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::CreateAssetAccount => (
                Some(View::CreateAssetAccountDialog(
                    super::create_asset_account::CreateAssetAccountDialog::default(),
                )),
                iced::Task::none(),
            ),
            Message::AccountView(account) => {
                let (view, task) =
                    super::account::Account::fetch(finance_manager.clone(), account.id());
                (
                    None,
                    iced::Task::done(AppMessage::SwitchView(View::ViewAccount(view)))
                        .chain(task.map(AppMessage::ViewAccountMessage)),
                )
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let account_table = utils::TableView::new(self.accounts.clone(), |account| {
            [
                utils::link(widget::text(account.0.name().to_string()))
                    .on_press(Message::AccountView(account.0.clone()))
                    .into(),
                utils::colored_currency_display(&account.1),
            ]
        })
        .sort_by(|a, b, column| match column {
            0 => a.0.name().cmp(b.0.name()),
            1 => a.1.cmp(&b.1),
            _ => panic!(),
        })
        .columns_sortable([true, true])
        .headers(["Account".to_string(), "Current Value".to_string()]);

        widget::column![
            utils::heading("Asset Account Overview", utils::HeadingLevel::H1),
            widget::row![widget::button("New Asset Account").on_press(Message::CreateAssetAccount)],
            widget::horizontal_rule(10),
            account_table.into_element(),
        ]
        .spacing(10)
        .into()
    }
}
