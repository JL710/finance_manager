use super::super::{utils, AppMessage, View};

use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Task<AppMessage> {
    let (view, task) = BookCheckingAccountOverview::new(finance_manager.clone());
    iced::Task::done(AppMessage::SwitchView(View::BookCheckingAccountOverview(
        view,
    )))
    .chain(task.map(AppMessage::BookCheckingAccountOverviewMessage))
}

pub enum Action {
    None,
    ViewAccount(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewAccount(fm_core::Id),
    Initialize(Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>),
}

#[derive(Debug, Clone)]
pub struct BookCheckingAccountOverview {
    accounts: Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>,
}

impl BookCheckingAccountOverview {
    pub fn new(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self {
                accounts: Vec::new(),
            },
            iced::Task::future(async move {
                let locked_manager = finance_manager.lock().await;
                let accounts = locked_manager
                    .get_accounts()
                    .await
                    .unwrap()
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
                        .await
                        .unwrap();
                    accounts_with_sums.push((account, sum));
                }

                Message::Initialize(accounts_with_sums)
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Action {
        match message {
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::Initialize(accounts) => {
                self.accounts = accounts;
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            utils::heading("Book Checking Account Overview", utils::HeadingLevel::H1),
            utils::TableView::new(self.accounts.clone(), |(account, sum)| {
                [
                    utils::link(widget::text(account.name().to_string()))
                        .on_press(Message::ViewAccount(account.id()))
                        .into(),
                    widget::text!("{}", sum).into(),
                ]
            })
            .headers(["Account".to_string(), "Sum".to_string()])
            .into_element()
        ]
        .spacing(10)
        .width(iced::Length::Fill)
        .into()
    }
}
