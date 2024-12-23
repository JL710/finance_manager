use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    None,
    ViewAccount(fm_core::Id),
    CreateNewAccount,
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewAccount(fm_core::Id),
    Initialize(Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>),
    AccountTable(utils::table_view::InnerMessage<Message>),
    New,
}

#[derive(Debug)]
pub struct BookCheckingAccountOverview {
    accounts_table:
        utils::table_view::State<(fm_core::account::BookCheckingAccount, fm_core::Currency), (), 2>,
}

impl BookCheckingAccountOverview {
    pub fn new(accounts: Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>) -> Self {
        Self {
            accounts_table: utils::table_view::State::new(accounts, ())
                .sort_by(|a, b, column| match column {
                    0 => b.0.name().cmp(a.0.name()),
                    1 => a.1.cmp(&b.1),
                    _ => std::cmp::Ordering::Equal,
                })
                .sortable_columns([true, true]),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new()),
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
                        .get_account_sum(&account.clone().into(), time::OffsetDateTime::now_utc())
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
        _finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::Initialize(accounts) => {
                self.accounts_table.set_items(accounts);
                Action::None
            }
            Message::New => Action::CreateNewAccount,
            Message::AccountTable(inner) => {
                if let utils::table_view::Action::OuterMessage(m) =
                    self.accounts_table.perform(inner)
                {
                    return self.update(m, _finance_manager);
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            utils::heading("Book Checking Account Overview", utils::HeadingLevel::H1),
            widget::button("Create new account").on_press(Message::New),
            widget::horizontal_rule(10),
            utils::table_view::table_view(&self.accounts_table)
                .headers(["Account".to_string(), "Sum".to_string()])
                .view(|(account, sum), _| [
                    utils::link(widget::text(account.name().to_string()))
                        .on_press(Message::ViewAccount(account.id()))
                        .into(),
                    utils::colored_currency_display(sum),
                ])
                .map(Message::AccountTable)
        ]
        .spacing(10)
        .height(iced::Fill)
        .width(iced::Fill)
        .into()
    }
}
