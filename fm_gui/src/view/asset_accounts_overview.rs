use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    None,
    CreateAssetAccount,
    /// could be any kind of account type
    ViewAccount(fm_core::Id),
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateAssetAccount,
    AccountView(fm_core::account::AssetAccount),
    Initialize(Vec<(fm_core::account::AssetAccount, fm_core::Currency)>),
    TableView(utils::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct View {
    account_table:
        utils::table_view::State<(fm_core::account::AssetAccount, fm_core::Currency), ()>,
}

impl std::default::Default for View {
    fn default() -> Self {
        View::new(Vec::new())
    }
}

impl View {
    pub fn new(accounts: Vec<(fm_core::account::AssetAccount, fm_core::Currency)>) -> Self {
        Self {
            account_table: utils::table_view::State::new(accounts, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name().cmp(b.0.name()),
                    1 => a.1.cmp(&b.1),
                    _ => panic!(),
                })
                .sortable_columns([0, 1]),
        }
    }

    pub fn fetch(
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            utils::failing_task(async move {
                let locked_manager = finance_manager.lock().await;
                let accounts = locked_manager
                    .get_accounts()
                    .await?
                    .iter()
                    .filter_map(|x| match &x {
                        fm_core::account::Account::AssetAccount(acc) => Some(acc.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let mut tuples = Vec::new();
                for account in accounts {
                    let amount = locked_manager
                        .get_account_sum(&account.clone().into(), time::OffsetDateTime::now_utc())
                        .await?;
                    tuples.push((account, amount));
                }
                Ok(Message::Initialize(tuples))
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
                self.account_table.set_items(accounts);
            }
            Message::CreateAssetAccount => return Action::CreateAssetAccount,
            Message::AccountView(account) => return Action::ViewAccount(account.id()),
            Message::TableView(m) => match self.account_table.perform(m) {
                utils::table_view::Action::OuterMessage(m) => {
                    return self.update(m, _finance_manager);
                }
                utils::table_view::Action::Task(task) => {
                    return Action::Task(task.map(Message::TableView));
                }
                _ => {}
            },
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let account_table = utils::table_view::table_view(&self.account_table)
            .headers(["Account".to_string(), "Current Value".to_string()])
            .view(|account, _| {
                [
                    utils::link(widget::text(account.0.name().to_string()))
                        .on_press(Message::AccountView(account.0.clone()))
                        .into(),
                    utils::colored_currency_display(&account.1),
                ]
            })
            .map(Message::TableView);

        super::view(
            "Asset Account Overview",
            utils::spaced_column![
                widget::row![utils::button::new(
                    "New Asset Account",
                    Some(Message::CreateAssetAccount)
                )],
                widget::horizontal_rule(10),
                account_table,
            ]
            .height(iced::Fill),
        )
    }
}
