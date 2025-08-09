use anyhow::Result;

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
    Reload(Vec<(fm_core::account::AssetAccount, fm_core::Currency)>),
    TableView(components::table_view::InnerMessage<Message>),
}

#[derive(Debug)]
pub struct View {
    account_table:
        components::table_view::State<(fm_core::account::AssetAccount, fm_core::Currency), ()>,
}

impl std::default::Default for View {
    fn default() -> Self {
        View::new(Vec::new())
    }
}

impl View {
    pub fn new(accounts: Vec<(fm_core::account::AssetAccount, fm_core::Currency)>) -> Self {
        Self {
            account_table: components::table_view::State::new(accounts, ())
                .sort_by(|a, b, column| match column {
                    0 => a.0.name.cmp(&b.0.name),
                    1 => a.1.cmp(&b.1),
                    _ => panic!(),
                })
                .sortable_columns([0, 1]),
        }
    }

    pub fn reload(
        &mut self,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> iced::Task<Message> {
        error::failing_task(init_future(finance_controller)).map(Message::Reload)
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            error::failing_task(init_future(finance_controller)).map(Message::Initialize),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::Reload(accounts) => {
                self.account_table.edit_items(|items| *items = accounts);
            }
            Message::Initialize(accounts) => {
                self.account_table.set_items(accounts);
            }
            Message::CreateAssetAccount => return Action::CreateAssetAccount,
            Message::AccountView(account) => return Action::ViewAccount(account.id),
            Message::TableView(m) => match self.account_table.perform(m) {
                components::table_view::Action::OuterMessage(m) => {
                    return self.update(m, _finance_controller);
                }
                components::table_view::Action::Task(task) => {
                    return Action::Task(task.map(Message::TableView));
                }
                _ => {}
            },
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let account_table = components::table_view::table_view(&self.account_table)
            .headers(["Account".to_string(), "Current Value".to_string()])
            .view(|account, _| {
                [
                    components::link(account.0.name.as_str())
                        .on_press(Message::AccountView(account.0.clone()))
                        .into(),
                    components::colored_currency_display(&account.1),
                ]
            })
            .map(Message::TableView);

        components::overlap_bottom_right(
            account_table,
            components::button::large_round_plus_button(Some(Message::CreateAssetAccount)),
        )
        .height(iced::Fill)
        .into()
    }
}

async fn init_future(
    finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
) -> Result<Vec<(fm_core::account::AssetAccount, fm_core::Currency)>> {
    let accounts = finance_controller
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
        let amount = finance_controller
            .get_account_sum(&account.clone().into(), time::OffsetDateTime::now_utc())
            .await?;
        tuples.push((account, amount));
    }
    Ok(tuples)
}
