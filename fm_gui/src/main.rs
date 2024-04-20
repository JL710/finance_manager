use fm_core;

use iced::advanced::Application;

mod table;
mod utils;
mod view;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum AppMessage {
    SwitchView(View),
    BudgetOverViewMessage(view::budget_overview::Message),
    SwitchToBudgetOverview,
    SwitchToCreateTransActionView,
    SwitchToAssetAccountsView,
    CreateAssetAccountMessage(view::create_asset_account::Message),
    CreateBudgetViewMessage(view::create_budget::Message),
    CreateTransactionViewMessage(view::create_transaction::Message),
    AssetAccountsMessage(view::show_asset_accounts::Message),
    ViewAccountMessage(view::view_account::Message),
}

#[derive(Debug, Clone)]
enum View {
    Empty,
    BudgetOverview(view::budget_overview::BudgetOverview),
    CreateAssetAccountDialog(view::create_asset_account::CreateAssetAccountDialog),
    CreateBudgetView(view::create_budget::CreateBudgetView),
    CreateTransactionView(view::create_transaction::CreateTransactionView),
    AssetAccounts(view::show_asset_accounts::AssetAccountOverview),
    ViewAccount(view::view_account::ViewAccount),
}

pub struct App {
    finance_manager: Arc<Mutex<fm_core::ram_finance_manager::RamFinanceManager>>,
    current_view: View,
}

impl Application for App {
    type Message = AppMessage;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let finance_manager = fm_core::ram_finance_manager::RamFinanceManager::new();
        (
            App {
                current_view: View::Empty,
                finance_manager: Arc::new(Mutex::new(finance_manager)),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Finance Manager")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::SwitchView(view) => self.current_view = view,
            AppMessage::BudgetOverViewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::BudgetOverview(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwitchToBudgetOverview => {
                return view::budget_overview::switch_view_command(self.finance_manager.clone())
            }
            AppMessage::CreateAssetAccountMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CreateAssetAccountDialog(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::CreateBudgetViewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CreateBudgetView(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::CreateTransactionViewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CreateTransactionView(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwitchToCreateTransActionView => {
                return view::create_transaction::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::AssetAccountsMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::AssetAccounts(ref mut view) => view.update(m),
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwitchToAssetAccountsView => {
                return view::show_asset_accounts::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::ViewAccountMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::ViewAccount(ref mut view) => view.update(m, self.finance_manager.clone()),
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            _ => {
                todo!()
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        iced::widget::row![
            iced::widget::column![
                iced::widget::button("AssetAccounts")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToAssetAccountsView),
                iced::widget::button("Transactions")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchView(View::Empty)),
                iced::widget::button("Budgets")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToBudgetOverview),
                iced::widget::button("Create Transaction")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToCreateTransActionView)
            ]
            .align_items(iced::Alignment::Start)
            .spacing(10)
            .width(iced::Length::FillPortion(2)),
            iced::widget::vertical_rule(5),
            iced::widget::column![match self.current_view {
                View::Empty => iced::widget::text("comming soon").into(),
                View::BudgetOverview(ref view) =>
                    view.view().map(AppMessage::BudgetOverViewMessage),
                View::CreateAssetAccountDialog(ref view) =>
                    view.view().map(AppMessage::CreateAssetAccountMessage),
                View::CreateBudgetView(ref view) =>
                    view.view().map(AppMessage::CreateBudgetViewMessage),
                View::CreateTransactionView(ref view) =>
                    view.view().map(AppMessage::CreateTransactionViewMessage),
                View::AssetAccounts(ref view) => view.view().map(AppMessage::AssetAccountsMessage),
                View::ViewAccount(ref view) => view.view().map(AppMessage::ViewAccountMessage),
            }]
            .width(iced::Length::FillPortion(9))
        ]
        .into()
    }
}

fn main() {
    App::run(iced::Settings::default()).unwrap();
}
