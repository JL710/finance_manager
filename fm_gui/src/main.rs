use iced::advanced::Application;

mod finance_managers;
mod table;
mod table_view;
mod timespan_input;
mod utils;
mod view;

use async_std::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AppMessage {
    SwitchView(View),
    BudgetOverViewMessage(view::budget_overview::Message),
    SwitchToBudgetOverview,
    SwitchToCreateTransActionView,
    SwitchToAssetAccountsView,
    SwitchToCategoryOverview,
    SwitchToBookCheckingAccountOverview,
    SwtichToSettingsView,
    CreateAssetAccountMessage(view::create_asset_account::Message),
    CreateBudgetViewMessage(view::create_budget::Message),
    CreateTransactionViewMessage(view::create_transaction::Message),
    AssetAccountsMessage(view::show_asset_accounts::Message),
    ViewAccountMessage(view::view_account::Message),
    TransactionViewMessage(view::view_transaction::Message),
    ViewBudgetMessage(view::view_budget::Message),
    CreateCategoryMessage(view::create_category::Message),
    CategoryOverviewMessage(view::category_overview::Message),
    ViewCategoryMessage(view::view_category::Message),
    BookCheckingAccountOverviewMessage(view::book_checking_account_overview::Message),
    CreateBookCheckingAccountMessage(view::create_book_checking_account::Message),
    SettingsMessage(view::settings::Message),
    ChangeFM(Arc<Mutex<finance_managers::FinanceManagers>>),
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
    TransactionView(view::view_transaction::TransactionView),
    ViewBudgetView(view::view_budget::BudgetView),
    CreateCategory(view::create_category::CreateCategory),
    CategoryOverview(view::category_overview::CategoryOverview),
    ViewCategory(view::view_category::ViewCategory),
    BookCheckingAccountOverview(view::book_checking_account_overview::BookCheckingAccountOverview),
    CreateBookCheckingAccount(view::create_book_checking_account::CreateBookCheckingAccount),
    Settings(view::settings::SettingsView),
}

pub struct App {
    finance_manager: Arc<Mutex<finance_managers::FinanceManagers>>,
    current_view: View,
}

impl Application for App {
    type Message = AppMessage;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let finance_manager = finance_managers::FinanceManagers::Server(
            fm_server::client::Client::new(String::from("http://localhost:3000")),
        );
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
                    View::AssetAccounts(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
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
            AppMessage::TransactionViewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::TransactionView(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::ViewBudgetMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::ViewBudgetView(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::CreateCategoryMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CreateCategory(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::CategoryOverviewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CategoryOverview(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwitchToCategoryOverview => {
                return view::category_overview::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::ViewCategoryMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::ViewCategory(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::BookCheckingAccountOverviewMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::BookCheckingAccountOverview(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwitchToBookCheckingAccountOverview => {
                return view::book_checking_account_overview::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::CreateBookCheckingAccountMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::CreateBookCheckingAccount(ref mut view) => {
                        view.update(m, self.finance_manager.clone())
                    }
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::SwtichToSettingsView => {
                self.current_view = View::Settings(view::settings::SettingsView::new(
                    self.finance_manager.clone(),
                ));
            }
            AppMessage::SettingsMessage(m) => {
                let (new_view, cmd) = match self.current_view {
                    View::Settings(ref mut view) => view.update(m),
                    _ => panic!(),
                };
                if let Some(new_view) = new_view {
                    self.current_view = new_view;
                }
                return cmd;
            }
            AppMessage::ChangeFM(fm) => {
                self.finance_manager = fm;
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        iced::widget::row![
            iced::widget::column![
                iced::widget::button("AssetAccounts")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToAssetAccountsView),
                iced::widget::button("BookCheckingAccounts")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToBookCheckingAccountOverview),
                iced::widget::button("Budgets")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToBudgetOverview),
                iced::widget::button("Categories")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToCategoryOverview),
                iced::widget::horizontal_rule(5),
                iced::widget::button("Create Transaction")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToCreateTransActionView),
                iced::widget::vertical_space(),
                iced::widget::button("Settings")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwtichToSettingsView),
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
                View::TransactionView(ref view) =>
                    view.view().map(AppMessage::TransactionViewMessage),
                View::ViewBudgetView(ref view) => view.view().map(AppMessage::ViewBudgetMessage),
                View::CreateCategory(ref view) =>
                    view.view().map(AppMessage::CreateCategoryMessage),
                View::CategoryOverview(ref view) =>
                    view.view().map(AppMessage::CategoryOverviewMessage),
                View::ViewCategory(ref view) => view.view().map(AppMessage::ViewCategoryMessage),
                View::BookCheckingAccountOverview(ref view) => view
                    .view()
                    .map(AppMessage::BookCheckingAccountOverviewMessage),
                View::CreateBookCheckingAccount(ref view) => view
                    .view()
                    .map(AppMessage::CreateBookCheckingAccountMessage),
                View::Settings(ref view) => view.view().map(AppMessage::SettingsMessage),
            }]
            .width(iced::Length::FillPortion(9))
        ]
        .into()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Nord
    }
}

fn main() {
    App::run(iced::Settings::default()).unwrap();
}
