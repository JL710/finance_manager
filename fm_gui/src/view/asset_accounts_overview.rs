use super::super::utils;

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    None,
    CreateAssetAccount,
    /// could be any kind of account type
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateAssetAccount,
    AccountView(fm_core::account::AssetAccount),
    Initialize(Vec<(fm_core::account::AssetAccount, fm_core::Currency)>),
}

#[derive(Debug, Clone, Default)]
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

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            iced::Task::future(async move {
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
                        .get_account_sum(&account.clone().into(), time::OffsetDateTime::now_utc())
                        .await
                        .unwrap();
                    tuples.push((account, amount));
                }
                Message::Initialize(tuples)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Initialize(accounts) => {
                self.accounts = accounts;
                Action::None
            }
            Message::CreateAssetAccount => Action::CreateAssetAccount,
            Message::AccountView(account) => Action::ViewAccount(account.id()),
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
            account_table,
        ]
        .spacing(10)
        .height(iced::Fill)
        .into()
    }
}
