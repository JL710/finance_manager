mod finance_managers;
mod utils;
mod view;

use async_std::sync::Mutex;
use iced::widget;
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

macro_rules! message_match_action {
    ($app:expr, $m:expr, $v:path) => {
        match $app.current_view {
            $v(ref mut view) => view.update($m, $app.finance_manager.clone()),
            _ => panic!(),
        }
    };
}

#[derive(Debug, Clone)]
pub enum AppMessage {
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
    AssetAccountsMessage(view::asset_accounts_overview::Message),
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

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum View {
    Empty,
    BudgetOverview(view::budget_overview::BudgetOverview),
    CreateAssetAccountDialog(view::create_asset_account::CreateAssetAccountDialog),
    CreateBudgetView(view::create_budget::CreateBudgetView),
    CreateTransactionView(view::create_transaction::CreateTransactionView),
    AssetAccounts(view::asset_accounts_overview::AssetAccountOverview),
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
            AppMessage::BudgetOverViewMessage(m) => {
                match message_match_action!(self, m, View::BudgetOverview) {
                    view::budget_overview::Action::None => {}
                    view::budget_overview::Action::ViewBudget(id) => {
                        return self.switch_view_budget(id);
                    }
                    view::budget_overview::Action::CreateBudget => {
                        self.current_view = View::CreateBudgetView(
                            view::create_budget::CreateBudgetView::default(),
                        );
                    }
                }
            }
            AppMessage::SwitchToBudgetOverview => {
                return self.switch_view_budget_overview();
            }
            AppMessage::CreateAssetAccountMessage(m) => {
                match message_match_action!(self, m, View::CreateAssetAccountDialog) {
                    view::create_asset_account::Action::AssetAccountCreated(id) => {
                        return self.switch_view_account(id)
                    }
                    view::create_asset_account::Action::None => {}
                    view::create_asset_account::Action::Task(t) => {
                        return t.map(AppMessage::CreateAssetAccountMessage);
                    }
                }
            }
            AppMessage::CreateBudgetViewMessage(m) => {
                match message_match_action!(self, m, View::CreateBudgetView) {
                    view::create_budget::Action::BudgetCreated(id) => {
                        return self.switch_view_budget(id);
                    }
                    view::create_budget::Action::None => {}
                    view::create_budget::Action::Task(t) => {
                        return t.map(AppMessage::CreateBudgetViewMessage);
                    }
                }
            }
            AppMessage::CreateTransactionViewMessage(m) => {
                match message_match_action!(self, m, View::CreateTransactionView) {
                    view::create_transaction::Action::TransactionCreated(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::create_transaction::Action::None => {}
                    view::create_transaction::Action::Task(t) => {
                        return t.map(AppMessage::CreateTransactionViewMessage);
                    }
                }
            }
            AppMessage::SwitchToCreateTransActionView => {
                return self.switch_view_transaction_create();
            }
            AppMessage::AssetAccountsMessage(m) => {
                match message_match_action!(self, m, View::AssetAccounts) {
                    view::asset_accounts_overview::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                    view::asset_accounts_overview::Action::CreateAssetAccount => {
                        self.current_view = View::CreateAssetAccountDialog(
                            view::create_asset_account::CreateAssetAccountDialog::default(),
                        );
                    }
                    view::asset_accounts_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToAssetAccountsView => {
                return self.switch_view_asset_account_overview();
            }
            AppMessage::ViewAccountMessage(m) => {
                match message_match_action!(self, m, View::ViewAccount) {
                    view::account::Action::Task(t) => {
                        return t.map(AppMessage::ViewAccountMessage);
                    }
                    view::account::Action::None => {}
                    view::account::Action::EditAssetAccount(acc) => {
                        let (v, t) = view::create_asset_account::CreateAssetAccountDialog::fetch(
                            acc.id(),
                            self.finance_manager.clone(),
                        );
                        self.current_view = View::CreateAssetAccountDialog(v);
                        return t.map(AppMessage::CreateAssetAccountMessage);
                    }
                    view::account::Action::EditBookCheckingAccount(acc) => {
                        let (v, t) =
                            view::create_book_checking_account::CreateBookCheckingAccount::fetch(
                                self.finance_manager.clone(),
                                acc.id(),
                            );
                        self.current_view = View::CreateBookCheckingAccount(v);
                        return t.map(AppMessage::CreateBookCheckingAccountMessage);
                    }
                    view::account::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::account::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                }
            }
            AppMessage::TransactionViewMessage(m) => {
                match message_match_action!(self, m, View::TransactionView) {
                    view::transaction::Action::None => {}
                    view::transaction::Action::Edit(id) => {
                        let (v, t) = view::create_transaction::CreateTransactionView::fetch(
                            self.finance_manager.clone(),
                            id,
                        );
                        self.current_view = View::CreateTransactionView(v);
                        return t.map(AppMessage::CreateTransactionViewMessage);
                    }
                    view::transaction::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                    view::transaction::Action::Delete(task) => {
                        return task.map(|_| AppMessage::SwitchToFilterTransactionView);
                    }
                    view::transaction::Action::ViewBudget(id) => {
                        return self.switch_view_budget(id);
                    }
                }
            }
            AppMessage::ViewBudgetMessage(m) => {
                match message_match_action!(self, m, View::ViewBudgetView) {
                    view::budget::Action::None => {}
                    view::budget::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::budget::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                    view::budget::Action::Edit(id) => {
                        return self.switch_view_budget_edit(id);
                    }
                    view::budget::Action::Task(t) => {
                        return t.map(AppMessage::ViewBudgetMessage);
                    }
                }
            }
            AppMessage::CreateCategoryMessage(m) => {
                match message_match_action!(self, m, View::CreateCategory) {
                    view::create_category::Action::CategoryCreated(id) => {
                        return self.switch_view_category(id);
                    }
                    view::create_category::Action::None => {}
                    view::create_category::Action::Task(t) => {
                        return t.map(AppMessage::CreateCategoryMessage);
                    }
                }
            }
            AppMessage::CategoryOverviewMessage(m) => {
                match message_match_action!(self, m, View::CategoryOverview) {
                    view::category_overview::Action::ViewCategory(id) => {
                        return self.switch_view_category(id);
                    }
                    view::category_overview::Action::NewCategory => {
                        self.current_view =
                            View::CreateCategory(view::create_category::CreateCategory::default());
                    }
                    view::category_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToCategoryOverview => {
                return self.switch_view_category_overview();
            }
            AppMessage::ViewCategoryMessage(m) => {
                match message_match_action!(self, m, View::ViewCategory) {
                    view::category::Action::Task(t) => {
                        return t.map(AppMessage::ViewCategoryMessage);
                    }
                    view::category::Action::None => {}
                    view::category::Action::EditCategory(id) => {
                        return self.switch_view_category_edit(id);
                    }
                    view::category::Action::DeleteCategory(task) => {
                        return task.map(|_| AppMessage::SwitchToCategoryOverview);
                    }
                    view::category::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::category::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                }
            }
            AppMessage::BookCheckingAccountOverviewMessage(m) => {
                match message_match_action!(self, m, View::BookCheckingAccountOverview) {
                    view::book_checking_account_overview::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                    view::book_checking_account_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToBookCheckingAccountOverview => {
                return self.switch_view_book_checking_account_overview();
            }
            AppMessage::CreateBookCheckingAccountMessage(m) => {
                match message_match_action!(self, m, View::CreateBookCheckingAccount) {
                    view::create_book_checking_account::Action::Task(t) => {
                        return t.map(AppMessage::CreateBookCheckingAccountMessage);
                    }
                    view::create_book_checking_account::Action::AccountCreated(id) => {
                        return self.switch_view_account(id);
                    }
                    view::create_book_checking_account::Action::None => {}
                }
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
                match message_match_action!(self, m, View::FilterTransaction) {
                    view::filter_transactions::Action::None => {}
                    view::filter_transactions::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::filter_transactions::Action::ViewAccount(id) => {
                        return self.switch_view_account(id);
                    }
                    view::filter_transactions::Action::Task(t) => {
                        return t.map(AppMessage::FilterTransactionMessage);
                    }
                }
            }
            AppMessage::CreateBillMessage(m) => {
                match message_match_action!(self, m, View::CreateBill) {
                    view::create_bill::Action::BillCreated(id) => {
                        return self.switch_view_bill(id);
                    }
                    view::create_bill::Action::None => {}
                    view::create_bill::Action::Task(t) => {
                        return t.map(AppMessage::CreateBillMessage);
                    }
                }
            }
            AppMessage::BillOverviewMessage(m) => {
                match message_match_action!(self, m, View::BillOverview) {
                    view::bill_overview::Action::ViewBill(id) => {
                        return self.switch_view_bill(id);
                    }
                    view::bill_overview::Action::NewBill => {
                        self.current_view =
                            View::CreateBill(view::create_bill::CreateBillView::default());
                    }
                    view::bill_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToFilterTransactionView => {
                self.current_view = View::Empty;
                return self.switch_view_transaction_filter();
            }
            AppMessage::SwitchToBillOverview => {
                return self.switch_view_bill_overview();
            }
            AppMessage::ViewBillMessage(m) => {
                match message_match_action!(self, m, View::ViewBill) {
                    view::bill::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::bill::Action::Edit(id) => {
                        return self.switch_view_bill_edit(id);
                    }
                    view::bill::Action::None => {}
                }
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<AppMessage> {
        fn icon_menu_item<'a>(
            text: &'a str,
            icon: &'static [u8],
            message: AppMessage,
        ) -> iced::Element<'a, AppMessage> {
            widget::button(
                widget::row![
                    text,
                    widget::horizontal_space(),
                    widget::Svg::new(widget::svg::Handle::from_memory(icon)).width(iced::Shrink)
                ]
                .align_y(iced::Center)
                .spacing(10),
            )
            .width(iced::Length::Fill)
            .on_press(message)
            .into()
        }

        iced::widget::row![
            widget::column![
                icon_menu_item(
                    "AssetAccounts",
                    include_bytes!("assets/bank2.svg"),
                    AppMessage::SwitchToAssetAccountsView
                ),
                icon_menu_item(
                    "BookCheckingAccounts",
                    include_bytes!("assets/cash.svg"),
                    AppMessage::SwitchToBookCheckingAccountOverview
                ),
                icon_menu_item(
                    "Budgets",
                    include_bytes!("assets/piggy-bank-fill.svg"),
                    AppMessage::SwitchToBudgetOverview
                ),
                icon_menu_item(
                    "Categories",
                    include_bytes!("assets/bookmark-fill.svg"),
                    AppMessage::SwitchToCategoryOverview
                ),
                icon_menu_item(
                    "Transactions",
                    include_bytes!("assets/send-fill.svg"),
                    AppMessage::SwitchToFilterTransactionView
                ),
                icon_menu_item(
                    "Bills",
                    include_bytes!("assets/folder-fill.svg"),
                    AppMessage::SwitchToBillOverview
                ),
                widget::horizontal_rule(5),
                icon_menu_item(
                    "Create Transaction",
                    include_bytes!("assets/plus-circle-fill.svg"),
                    AppMessage::SwitchToCreateTransActionView
                ),
                widget::vertical_space(),
                icon_menu_item(
                    "Settings",
                    include_bytes!("assets/gear-fill.svg"),
                    AppMessage::SwitchToSettingsView
                ),
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

    fn switch_view_account(&mut self, account: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::account::Account::fetch(self.finance_manager.clone(), account);
        self.current_view = View::ViewAccount(view);
        task.map(AppMessage::ViewAccountMessage)
    }

    fn switch_view_asset_account_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::asset_accounts_overview::AssetAccountOverview::fetch(
            self.finance_manager.clone(),
        );
        self.current_view = View::AssetAccounts(view);
        task.map(AppMessage::AssetAccountsMessage)
    }

    fn switch_view_bill_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::bill_overview::BillOverview::fetch(self.finance_manager.clone());
        self.current_view = View::BillOverview(view);
        task.map(AppMessage::BillOverviewMessage)
    }

    fn switch_view_bill(&mut self, bill: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::bill::Bill::fetch(bill, self.finance_manager.clone());
        self.current_view = View::ViewBill(view);
        task.map(AppMessage::ViewBillMessage)
    }

    fn switch_view_book_checking_account_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::book_checking_account_overview::BookCheckingAccountOverview::fetch(
            self.finance_manager.clone(),
        );
        self.current_view = View::BookCheckingAccountOverview(view);
        task.map(AppMessage::BookCheckingAccountOverviewMessage)
    }

    fn switch_view_budget_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::budget_overview::BudgetOverview::fetch(self.finance_manager.clone());
        self.current_view = View::BudgetOverview(view);
        task.map(AppMessage::BudgetOverViewMessage)
    }

    fn switch_view_budget(&mut self, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::budget::Budget::fetch(budget, 0, self.finance_manager.clone());
        self.current_view = View::ViewBudgetView(view);
        task.map(AppMessage::ViewBudgetMessage)
    }

    fn switch_view_category_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::category_overview::CategoryOverview::fetch(self.finance_manager.clone());
        self.current_view = View::CategoryOverview(view);
        task.map(AppMessage::CategoryOverviewMessage)
    }

    fn switch_view_category(&mut self, category: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::category::Category::fetch(self.finance_manager.clone(), category);
        self.current_view = View::ViewCategory(view);
        task.map(AppMessage::ViewCategoryMessage)
    }

    fn switch_view_bill_edit(&mut self, bill: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_bill::CreateBillView::fetch(bill, self.finance_manager.clone());
        self.current_view = View::CreateBill(view);
        task.map(AppMessage::CreateBillMessage)
    }

    fn switch_view_budget_edit(&mut self, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_budget::CreateBudgetView::fetch(budget, self.finance_manager.clone());
        self.current_view = View::CreateBudgetView(view);
        task.map(AppMessage::CreateBudgetViewMessage)
    }

    fn switch_view_transaction_create(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_transaction::CreateTransactionView::new(self.finance_manager.clone());
        self.current_view = View::CreateTransactionView(view);
        task.map(AppMessage::CreateTransactionViewMessage)
    }

    fn switch_view_transaction_filter(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::filter_transactions::FilterTransactionView::new(self.finance_manager.clone());
        self.current_view = View::FilterTransaction(view);
        task.map(AppMessage::FilterTransactionMessage)
    }

    fn switch_view_transaction(&mut self, transaction: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::transaction::Transaction::fetch(transaction, self.finance_manager.clone());
        self.current_view = View::TransactionView(view);
        task.map(AppMessage::TransactionViewMessage)
    }

    fn switch_view_category_edit(&mut self, category: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_category::CreateCategory::fetch(category, self.finance_manager.clone());
        self.current_view = View::CreateCategory(view);
        task.map(AppMessage::CreateCategoryMessage)
    }
}

fn main() {
    iced::application("Finance Manager", App::update, App::view)
        .theme(|_| iced::Theme::Nord)
        .run()
        .unwrap();
}
