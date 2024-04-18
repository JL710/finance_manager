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
    /*CreateAssetAccountMessage(view::create_asset_account::Message),
    AccountOverviewMessage(view::show_asset_accounts::Message),
    CreateTransactionViewMessage(view::create_transaction::Message),
    CreateBudgetViewMessage(view::create_budget::Message),*/
}

#[derive(Debug, Clone)]
enum View {
    Empty,
    BudgetOverview(view::budget_overview::BudgetOverview),
    /*
    CreateBudgetView(view::create_budget::CreateBudgetView),
    CreateAssetAccountDialog(view::create_asset_account::CreateAssetAccountDialog),
    CreateTransactionView(view::create_transaction::CreateTransactionView),
    ViewAccount(view::view_account::ViewAccount),
    ShowAssetAccounts(view::show_asset_accounts::AssetAccountOverview),*/
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
            /*AppMessage::AssetAccountOverview => {
                self.current_view = Box::new(view::show_asset_accounts::AssetAccountOverview::new(
                    &self.finance_manager,
                ))
            }
            AppMessage::BudgetOverview => {
                self.current_view = Box::new(view::budget_overview::BudgetOverview::new(
                    &self.finance_manager,
                ))
            }
            AppMessage::TransactionOverview => {
                self.current_view = Box::new(view::comming_soon_view::EmptyView {})
            }
            AppMessage::CreateTransactionView => {
                self.current_view = Box::new(view::create_transaction::CreateTransactionView::new(
                    &self.finance_manager,
                ))
            }*/
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
                    .on_press(AppMessage::SwitchView(View::Empty)),
                iced::widget::button("Transactions")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchView(View::Empty)),
                iced::widget::button("Budgets")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToBudgetOverview),
                iced::widget::button("Create Transaction")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchView(View::Empty))
            ]
            .align_items(iced::Alignment::Start)
            .spacing(10)
            .width(iced::Length::FillPortion(2)),
            iced::widget::vertical_rule(5),
            iced::widget::column![match self.current_view {
                View::Empty => iced::widget::text("comming soon").into(),
                View::BudgetOverview(ref view) =>
                    view.view().map(AppMessage::BudgetOverViewMessage),
            }]
            .width(iced::Length::FillPortion(9))
        ]
        .into()
    }
}

fn main() {
    App::run(iced::Settings::default()).unwrap();
}
