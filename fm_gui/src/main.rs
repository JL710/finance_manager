mod finance_managers;
mod utils;
mod view;

use async_std::sync::Mutex;
use std::sync::Arc;

macro_rules! message_match {
    ($app:expr, $m:expr, $v:path) => {
        let (new_view, cmd) = match $app.current_view {
            $v(ref mut view) => view.update($m, $app.finance_manager.clone()),
            _ => panic!(),
        };
        if let Some(new_view) = new_view {
            $app.current_view = new_view;
        }
        return cmd;
    };
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    SwitchView(View),
    BudgetOverViewMessage(view::budget_overview::Message),
    SwitchToBudgetOverview,
    SwitchToCreateTransActionView,
    SwitchToAssetAccountsView,
    SwitchToCategoryOverview,
    SwitchToBookCheckingAccountOverview,
    SwitchToSettingsView,
    SwitchToFilterTransactionView,
    SwitchToBillOverview,
    CreateAssetAccountMessage(view::create_asset_account::Message),
    CreateBudgetViewMessage(view::create_budget::Message),
    CreateTransactionViewMessage(view::create_transaction::Message),
    AssetAccountsMessage(view::show_asset_accounts::Message),
    ViewAccountMessage(view::account::Message),
    TransactionViewMessage(view::transaction::Message),
    ViewBudgetMessage(view::budget::Message),
    CreateCategoryMessage(view::create_category::Message),
    CategoryOverviewMessage(view::category_overview::Message),
    ViewCategoryMessage(view::category::Message),
    BookCheckingAccountOverviewMessage(view::book_checking_account_overview::Message),
    CreateBookCheckingAccountMessage(view::create_book_checking_account::Message),
    SettingsMessage(view::settings::Message),
    ChangeFM(Arc<Mutex<finance_managers::FinanceManagers>>),
    FilterTransactionMessage(view::filter_transactions::Message),
    CreateBillMessage(view::create_bill::Message),
    BillOverviewMessage(view::bill_overview::Message),
    ViewBillMessage(view::bill::Message),
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum View {
    Empty,
    BudgetOverview(view::budget_overview::BudgetOverview),
    CreateAssetAccountDialog(view::create_asset_account::CreateAssetAccountDialog),
    CreateBudgetView(view::create_budget::CreateBudgetView),
    CreateTransactionView(view::create_transaction::CreateTransactionView),
    AssetAccounts(view::show_asset_accounts::AssetAccountOverview),
    ViewAccount(view::account::Account),
    TransactionView(view::transaction::Transaction),
    ViewBudgetView(view::budget::Budget),
    CreateCategory(view::create_category::CreateCategory),
    CategoryOverview(view::category_overview::CategoryOverview),
    ViewCategory(view::category::Category),
    BookCheckingAccountOverview(view::book_checking_account_overview::BookCheckingAccountOverview),
    CreateBookCheckingAccount(view::create_book_checking_account::CreateBookCheckingAccount),
    Settings(view::settings::SettingsView),
    FilterTransaction(view::filter_transactions::FilterTransactionView),
    CreateBill(view::create_bill::CreateBillView),
    BillOverview(view::bill_overview::BillOverview),
    ViewBill(view::bill::Bill),
}

pub struct App {
    finance_manager: Arc<Mutex<finance_managers::FinanceManagers>>,
    current_view: View,
}

impl Default for App {
    fn default() -> Self {
        let finance_manager = finance_managers::FinanceManagers::Server(
            fm_server::client::Client::new(String::from("http://localhost:3000")),
        );
        App {
            current_view: View::Empty,
            finance_manager: Arc::new(Mutex::new(finance_manager)),
        }
    }
}

impl App {
    fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::SwitchView(view) => self.current_view = view,
            AppMessage::BudgetOverViewMessage(m) => {
                message_match!(self, m, View::BudgetOverview);
            }
            AppMessage::SwitchToBudgetOverview => {
                return view::budget_overview::switch_view_command(self.finance_manager.clone())
            }
            AppMessage::CreateAssetAccountMessage(m) => {
                message_match!(self, m, View::CreateAssetAccountDialog);
            }
            AppMessage::CreateBudgetViewMessage(m) => {
                message_match!(self, m, View::CreateBudgetView);
            }
            AppMessage::CreateTransactionViewMessage(m) => {
                message_match!(self, m, View::CreateTransactionView);
            }
            AppMessage::SwitchToCreateTransActionView => {
                return view::create_transaction::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::AssetAccountsMessage(m) => {
                message_match!(self, m, View::AssetAccounts);
            }
            AppMessage::SwitchToAssetAccountsView => {
                return view::show_asset_accounts::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::ViewAccountMessage(m) => {
                message_match!(self, m, View::ViewAccount);
            }
            AppMessage::TransactionViewMessage(m) => {
                message_match!(self, m, View::TransactionView);
            }
            AppMessage::ViewBudgetMessage(m) => {
                message_match!(self, m, View::ViewBudgetView);
            }
            AppMessage::CreateCategoryMessage(m) => {
                message_match!(self, m, View::CreateCategory);
            }
            AppMessage::CategoryOverviewMessage(m) => {
                message_match!(self, m, View::CategoryOverview);
            }
            AppMessage::SwitchToCategoryOverview => {
                return view::category_overview::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::ViewCategoryMessage(m) => {
                message_match!(self, m, View::ViewCategory);
            }
            AppMessage::BookCheckingAccountOverviewMessage(m) => {
                message_match!(self, m, View::BookCheckingAccountOverview);
            }
            AppMessage::SwitchToBookCheckingAccountOverview => {
                return view::book_checking_account_overview::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::CreateBookCheckingAccountMessage(m) => {
                message_match!(self, m, View::CreateBookCheckingAccount);
            }
            AppMessage::SwitchToSettingsView => {
                self.current_view = View::Settings(view::settings::SettingsView::new(
                    self.finance_manager.clone(),
                ));
            }
            AppMessage::SettingsMessage(m) => {
                message_match!(self, m, View::Settings);
            }
            AppMessage::ChangeFM(fm) => {
                self.finance_manager = fm;
            }
            AppMessage::FilterTransactionMessage(m) => {
                message_match!(self, m, View::FilterTransaction);
            }
            AppMessage::CreateBillMessage(m) => {
                message_match!(self, m, View::CreateBill);
            }
            AppMessage::BillOverviewMessage(m) => {
                message_match!(self, m, View::BillOverview);
            }
            AppMessage::SwitchToFilterTransactionView => {
                self.current_view = View::Empty;
                return view::filter_transactions::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::SwitchToBillOverview => {
                return view::bill_overview::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::ViewBillMessage(m) => {
                message_match!(self, m, View::ViewBill);
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<AppMessage> {
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
                iced::widget::button("Transactions")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToFilterTransactionView),
                iced::widget::button("Bills")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToBillOverview),
                iced::widget::horizontal_rule(5),
                iced::widget::button("Create Transaction")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToCreateTransActionView),
                iced::widget::vertical_space(),
                iced::widget::button("Settings")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::SwitchToSettingsView),
            ]
            .align_x(iced::Alignment::Start)
            .spacing(10)
            .width(iced::Length::FillPortion(2)),
            iced::widget::vertical_rule(5),
            iced::widget::container(match self.current_view {
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
                View::FilterTransaction(ref view) =>
                    view.view().map(AppMessage::FilterTransactionMessage),
                View::CreateBill(ref view) => view.view().map(AppMessage::CreateBillMessage),
                View::BillOverview(ref view) => view.view().map(AppMessage::BillOverviewMessage),
                View::ViewBill(ref view) => view.view().map(AppMessage::ViewBillMessage),
            })
            .padding(10)
            .width(iced::Length::FillPortion(9))
        ]
        .into()
    }
}

fn main() {
    iced::application("Finance Manager", App::update, App::view)
        .theme(|_| iced::Theme::Nord)
        .run()
        .unwrap();
}
