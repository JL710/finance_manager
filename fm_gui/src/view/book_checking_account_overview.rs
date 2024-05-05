use super::super::{utils, AppMessage, View};

use anyhow::Result;
use iced::widget;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(
        async move {
            View::BookCheckingAccountOverview(
                BookCheckingAccountOverview::fetch(finance_manager)
                    .await
                    .unwrap(),
            )
        },
        AppMessage::SwitchView,
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub struct BookCheckingAccountOverview {
    accounts: Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>,
}

impl BookCheckingAccountOverview {
    pub fn new(accounts: Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>) -> Self {
        Self { accounts }
    }

    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let accounts = locked_manager
            .get_accounts()
            .await?
            .iter()
            .filter_map(|x| match x {
                fm_core::account::Account::BookCheckingAccount(x) => Some(x.clone()),
                _ => None,
            })
            .collect::<Vec<fm_core::account::BookCheckingAccount>>();
        let mut accounts_with_sums = Vec::new();
        for account in accounts {
            let sum = locked_manager
                .get_account_sum(&account.clone().into(), chrono::Utc::now())
                .await?;
            accounts_with_sums.push((account, sum));
        }
        Ok(Self::new(accounts_with_sums))
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::ViewAccount(id) => (
                None,
                super::view_account::switch_view_command(id, finance_manager),
            ),
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let table =
            super::super::table_view::TableView::new(self.accounts.clone(), |(account, sum)| {
                [
                    widget::button(widget::text(account.name().to_string()))
                        .style(utils::button_link_style)
                        .on_press(Message::ViewAccount(account.id()))
                        .into(),
                    widget::text(format!("{}", sum)).into(),
                ]
            })
            .headers(["Account".to_string(), "Sum".to_string()]);
        table.into_element()
    }
}
