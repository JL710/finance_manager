use iced::widget;

pub enum Action {
    None,
    ViewAccount(fm_core::Id),
    CreateNewAccount,
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    ViewAccount(fm_core::Id),
    Initialize(Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>),
    AccountTable(utils::table_view::InnerMessage<Message>),
    New,
}

#[derive(Debug)]
pub struct View {
    accounts_table:
        utils::table_view::State<(fm_core::account::BookCheckingAccount, fm_core::Currency), ()>,
}

impl View {
    pub fn new(accounts: Vec<(fm_core::account::BookCheckingAccount, fm_core::Currency)>) -> Self {
        Self {
            accounts_table: utils::table_view::State::new(accounts, ())
                .sort_by(|a, b, column| match column {
                    0 => b.0.name().cmp(a.0.name()),
                    1 => a.1.cmp(&b.1),
                    _ => std::cmp::Ordering::Equal,
                })
                .sortable_columns([0, 1]),
        }
    }

    pub fn fetch(
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::new(Vec::new()),
            utils::failing_task(async move {
                let accounts = finance_controller
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
                    let sum = finance_controller
                        .get_account_sum(&account.clone().into(), time::OffsetDateTime::now_utc())
                        .await?;
                    accounts_with_sums.push((account, sum));
                }

                Ok(Message::Initialize(accounts_with_sums))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
    ) -> Action {
        match message {
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::Initialize(accounts) => {
                self.accounts_table.set_items(accounts);
                Action::None
            }
            Message::New => Action::CreateNewAccount,
            Message::AccountTable(inner) => match self.accounts_table.perform(inner) {
                utils::table_view::Action::OuterMessage(m) => self.update(m, _finance_controller),
                utils::table_view::Action::Task(task) => {
                    Action::Task(task.map(Message::AccountTable))
                }
                _ => Action::None,
            },
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        super::view(
            "Book Checking Account Overview",
            utils::spaced_column![
                utils::button::new("Create new account", Some(Message::New)),
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
            .height(iced::Fill)
            .width(iced::Fill),
        )
    }
}
