mod finance_managers;
mod settings;
mod view;

use async_std::sync::Mutex;
use clap::Parser;
use iced::widget;
use std::sync::Arc;

use fm_core::FinanceManager;

macro_rules! message_match_action {
    ($app:expr, $m:expr, $v:path) => {
        match $app.current_view {
            $v(ref mut view) => view.update($m, $app.finance_manager.clone()),
            _ => {
                tracing::debug!("message not handled");
                return iced::Task::none();
            }
        }
    };
}

type Fm = Arc<Mutex<fm_core::FMController<finance_managers::FinanceManagers>>>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
enum View {
    Empty,
    Tutorial(Vec<widget::markdown::Item>),
    License,
    BudgetOverview(view::budget_overview::BudgetOverview),
    CreateAssetAccount(view::create_asset_account::CreateAssetAccountDialog),
    CreateBudget(view::create_budget::CreateBudgetView),
    CreateTransaction(view::create_transaction::CreateTransactionView),
    AssetAccounts(view::asset_accounts_overview::AssetAccountOverview),
    Account(view::account::Account),
    Transaction(view::transaction::Transaction),
    Budget(view::budget::Budget),
    CreateCategory(view::create_category::CreateCategory),
    CategoryOverview(view::category_overview::CategoryOverview),
    Category(view::category::Category),
    BookCheckingAccountOverview(view::book_checking_account_overview::BookCheckingAccountOverview),
    CreateBookCheckingAccount(view::create_book_checking_account::CreateBookCheckingAccount),
    Settings(view::settings::SettingsView),
    FilterTransaction(view::filter_transactions::FilterTransactionView),
    CreateBill(view::create_bill::CreateBillView),
    BillOverview(view::bill_overview::BillOverview),
    Bill(view::bill::Bill),
}

impl View {
    fn account(&mut self, finance_manager: Fm, account: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::account::Account::fetch(finance_manager, account);
        *self = Self::Account(view);
        task.map(ViewMessage::Account).map(AppMessage::ViewMessage)
    }

    fn asset_account_overview(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) =
            view::asset_accounts_overview::AssetAccountOverview::fetch(finance_manager);
        *self = Self::AssetAccounts(view);
        task.map(ViewMessage::AssetAccounts)
            .map(AppMessage::ViewMessage)
    }

    fn bill_overview(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) = view::bill_overview::BillOverview::fetch(finance_manager);
        *self = Self::BillOverview(view);
        task.map(ViewMessage::BillOverview)
            .map(AppMessage::ViewMessage)
    }

    fn bill(&mut self, finance_manager: Fm, bill: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::bill::Bill::fetch(bill, finance_manager);
        *self = Self::Bill(view);
        task.map(ViewMessage::Bill).map(AppMessage::ViewMessage)
    }

    fn book_checking_account_overview(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) = view::book_checking_account_overview::BookCheckingAccountOverview::fetch(
            finance_manager,
        );
        *self = Self::BookCheckingAccountOverview(view);
        task.map(ViewMessage::BookCheckingAccountOverview)
            .map(AppMessage::ViewMessage)
    }

    fn budget_overview(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) = view::budget_overview::BudgetOverview::fetch(finance_manager);
        *self = Self::BudgetOverview(view);
        task.map(ViewMessage::BudgetOverview)
            .map(AppMessage::ViewMessage)
    }

    fn budget(&mut self, finance_manager: Fm, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::budget::Budget::fetch(budget, 0, finance_manager);
        *self = Self::Budget(view);
        task.map(ViewMessage::Budget).map(AppMessage::ViewMessage)
    }

    fn category_overview(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) = view::category_overview::CategoryOverview::fetch(finance_manager);
        *self = View::CategoryOverview(view);
        task.map(ViewMessage::CategoryOverview)
            .map(AppMessage::ViewMessage)
    }

    fn category(&mut self, finance_manager: Fm, category: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::category::Category::fetch(finance_manager, category);
        *self = Self::Category(view);
        task.map(ViewMessage::Category).map(AppMessage::ViewMessage)
    }

    fn create_bill(
        &mut self,
        finance_manager: Fm,
        bill: Option<fm_core::Id>,
    ) -> iced::Task<AppMessage> {
        let (view, task) = view::create_bill::CreateBillView::fetch(bill, finance_manager);
        *self = Self::CreateBill(view);
        task.map(ViewMessage::CreateBill)
            .map(AppMessage::ViewMessage)
    }

    fn budget_edit(&mut self, finance_manager: Fm, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::create_budget::CreateBudgetView::fetch(budget, finance_manager);
        *self = Self::CreateBudget(view);
        task.map(ViewMessage::CreateBudget)
            .map(AppMessage::ViewMessage)
    }

    fn transaction_create(
        &mut self,
        finance_manager: Fm,
        id: Option<fm_core::Id>,
    ) -> iced::Task<AppMessage> {
        let (view, task) = if let Some(id) = id {
            view::create_transaction::CreateTransactionView::fetch(finance_manager, id)
        } else {
            view::create_transaction::CreateTransactionView::new(finance_manager)
        };
        *self = Self::CreateTransaction(view);
        task.map(ViewMessage::CreateTransaction)
            .map(AppMessage::ViewMessage)
    }

    fn transaction_filter(&mut self, finance_manager: Fm) -> iced::Task<AppMessage> {
        let (view, task) = view::filter_transactions::FilterTransactionView::new(finance_manager);
        *self = Self::FilterTransaction(view);
        task.map(ViewMessage::FilterTransaction)
            .map(AppMessage::ViewMessage)
    }

    fn transaction(
        &mut self,
        finance_manager: Fm,
        transaction: fm_core::Id,
    ) -> iced::Task<AppMessage> {
        let (view, task) = view::transaction::Transaction::fetch(transaction, finance_manager);
        *self = Self::Transaction(view);
        task.map(ViewMessage::Transaction)
            .map(AppMessage::ViewMessage)
    }

    fn category_edit(
        &mut self,
        finance_manager: Fm,
        category: fm_core::Id,
    ) -> iced::Task<AppMessage> {
        let (view, task) = view::create_category::CreateCategory::fetch(category, finance_manager);
        *self = Self::CreateCategory(view);
        task.map(ViewMessage::CreateCategory)
            .map(AppMessage::ViewMessage)
    }

    fn asset_account_edit(
        &mut self,
        finance_manager: Fm,
        id: fm_core::Id,
    ) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_asset_account::CreateAssetAccountDialog::fetch(id, finance_manager);
        *self = Self::CreateAssetAccount(view);
        task.map(ViewMessage::CreateAssetAccount)
            .map(AppMessage::ViewMessage)
    }

    fn book_checking_account_edit(
        &mut self,
        finance_manager: Fm,
        id: fm_core::Id,
    ) -> iced::Task<AppMessage> {
        let (view, task) = view::create_book_checking_account::CreateBookCheckingAccount::fetch(
            finance_manager,
            id,
        );
        *self = Self::CreateBookCheckingAccount(view);
        task.map(ViewMessage::CreateBookCheckingAccount)
            .map(AppMessage::ViewMessage)
    }

    fn new_bill_with_transaction(
        &mut self,
        finance_manager: Fm,
        transaction: fm_core::Transaction,
    ) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_bill::CreateBillView::new_with_transaction(finance_manager, transaction);
        *self = Self::CreateBill(view);
        task.map(ViewMessage::CreateBill)
            .map(AppMessage::ViewMessage)
    }
}

#[derive(Debug, Clone)]
enum ViewMessage {
    None,
    BudgetOverview(view::budget_overview::Message),
    CreateAssetAccount(view::create_asset_account::Message),
    CreateBudget(view::create_budget::Message),
    CreateTransaction(view::create_transaction::MessageContainer),
    AssetAccounts(view::asset_accounts_overview::Message),
    Account(view::account::MessageContainer),
    Transaction(view::transaction::MessageContainer),
    Budget(view::budget::MessageContainer),
    CreateCategory(view::create_category::Message),
    CategoryOverview(view::category_overview::Message),
    Category(view::category::Message),
    BookCheckingAccountOverview(view::book_checking_account_overview::Message),
    CreateBookCheckingAccount(view::create_book_checking_account::Message),
    Settings(view::settings::Message),
    FilterTransaction(view::filter_transactions::Message),
    CreateBill(view::create_bill::Message),
    BillOverview(view::bill_overview::Message),
    Bill(view::bill::MessageContainer),
}

#[derive(Debug, Clone)]
enum AppMessage {
    Ignore,
    ToggleSidebarCollapse,
    SwitchToBudgetOverview,
    SwitchToCreateTransActionView,
    SwitchToAssetAccountsView,
    SwitchToCategoryOverview,
    SwitchToBookCheckingAccountOverview,
    SwitchToSettingsView,
    SwitchToFilterTransactionView,
    SwitchToBillOverview,
    SwitchToLicense,
    ViewMessage(ViewMessage),
}

struct SvgCache {
    bank2: widget::svg::Handle,
    cash: widget::svg::Handle,
    piggy_bank_fill: widget::svg::Handle,
    bookmark_fill: widget::svg::Handle,
    send_fill: widget::svg::Handle,
    folder_fill: widget::svg::Handle,
    plus_circle_fill: widget::svg::Handle,
    gear_fill: widget::svg::Handle,
    book_fill: widget::svg::Handle,
    list: widget::svg::Handle,
}

impl Default for SvgCache {
    fn default() -> Self {
        SvgCache {
            bank2: widget::svg::Handle::from_memory(include_bytes!("../assets/bank2.svg")),
            cash: widget::svg::Handle::from_memory(include_bytes!("../assets/cash.svg")),
            piggy_bank_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/piggy-bank-fill.svg"
            )),
            bookmark_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/bookmark-fill.svg"
            )),
            send_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/send-fill.svg")),
            folder_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/folder-fill.svg"
            )),
            plus_circle_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/plus-circle-fill.svg"
            )),
            gear_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/gear-fill.svg")),
            book_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/book-fill.svg")),
            list: widget::svg::Handle::from_memory(include_bytes!("../assets/list.svg")),
        }
    }
}

pub struct App {
    finance_manager: Arc<Mutex<fm_core::FMController<finance_managers::FinanceManagers>>>,
    current_view: View,
    svg_cache: SvgCache,
    side_bar_collapsed: bool,
    settings: settings::Settings,
}

impl Default for App {
    fn default() -> Self {
        let finance_manager =
            fm_core::FMController::with_finance_manager(finance_managers::FinanceManagers::Ram(
                fm_core::managers::RamFinanceManager::new(()).unwrap(),
            ));
        App {
            current_view: View::Empty,
            finance_manager: Arc::new(Mutex::new(finance_manager)),
            svg_cache: SvgCache::default(),
            side_bar_collapsed: false,
            settings: settings::Settings::default(),
        }
    }
}

impl App {
    fn new(finance_manager: Fm, settings: settings::Settings) -> Self {
        App {
            current_view: View::Tutorial(
                widget::markdown::parse(include_str!("view/tutorial.md")).collect(),
            ),
            finance_manager,
            svg_cache: SvgCache::default(),
            side_bar_collapsed: false,
            settings,
        }
    }

    fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Ignore => {}
            AppMessage::ToggleSidebarCollapse => {
                self.side_bar_collapsed = !self.side_bar_collapsed;
            }
            AppMessage::SwitchToLicense => {
                self.current_view = View::License;
            }
            AppMessage::ViewMessage(view_message) => match view_message {
                ViewMessage::None => {}
                ViewMessage::BudgetOverview(m) => {
                    match message_match_action!(self, m, View::BudgetOverview) {
                        view::budget_overview::Action::None => {}
                        view::budget_overview::Action::ViewBudget(id) => {
                            return self.current_view.budget(self.finance_manager.clone(), id);
                        }
                        view::budget_overview::Action::CreateBudget => {
                            self.current_view = View::CreateBudget(
                                view::create_budget::CreateBudgetView::default(),
                            );
                        }
                        view::budget_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::BudgetOverview)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::CreateAssetAccount(m) => {
                    match message_match_action!(self, m, View::CreateAssetAccount) {
                        view::create_asset_account::Action::AssetAccountCreated(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id)
                        }
                        view::create_asset_account::Action::None => {}
                        view::create_asset_account::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateAssetAccount)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_asset_account::Action::Cancel => {
                            return self
                                .current_view
                                .asset_account_overview(self.finance_manager.clone())
                        }
                        view::create_asset_account::Action::CancelWithId(acc_id) => {
                            return self
                                .current_view
                                .account(self.finance_manager.clone(), acc_id);
                        }
                    }
                }
                ViewMessage::CreateBudget(m) => {
                    match message_match_action!(self, m, View::CreateBudget) {
                        view::create_budget::Action::BudgetCreated(id) => {
                            return self.current_view.budget(self.finance_manager.clone(), id);
                        }
                        view::create_budget::Action::None => {}
                        view::create_budget::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateBudget)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_budget::Action::Cancel => {
                            return self
                                .current_view
                                .budget_overview(self.finance_manager.clone());
                        }
                        view::create_budget::Action::CancelWithId(budget_id) => {
                            return self
                                .current_view
                                .budget(self.finance_manager.clone(), budget_id);
                        }
                    }
                }
                ViewMessage::CreateTransaction(m) => {
                    match message_match_action!(self, m, View::CreateTransaction) {
                        view::create_transaction::Action::TransactionCreated(id) => {
                            return self
                                .current_view
                                .transaction(self.finance_manager.clone(), id);
                        }
                        view::create_transaction::Action::None => {}
                        view::create_transaction::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateTransaction)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_transaction::Action::Cancel => {
                            return self
                                .current_view
                                .transaction_filter(self.finance_manager.clone());
                        }
                        view::create_transaction::Action::CancelWithId(transaction_id) => {
                            return self
                                .current_view
                                .transaction(self.finance_manager.clone(), transaction_id);
                        }
                    }
                }
                ViewMessage::AssetAccounts(m) => {
                    match message_match_action!(self, m, View::AssetAccounts) {
                        view::asset_accounts_overview::Action::ViewAccount(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id);
                        }
                        view::asset_accounts_overview::Action::CreateAssetAccount => {
                            self.current_view = View::CreateAssetAccount(
                                view::create_asset_account::CreateAssetAccountDialog::default(),
                            );
                        }
                        view::asset_accounts_overview::Action::None => {}
                        view::asset_accounts_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::AssetAccounts)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::Account(m) => match message_match_action!(self, m, View::Account) {
                    view::account::Action::Task(t) => {
                        return t.map(ViewMessage::Account).map(AppMessage::ViewMessage);
                    }
                    view::account::Action::None => {}
                    view::account::Action::EditAssetAccount(acc) => {
                        return self
                            .current_view
                            .asset_account_edit(self.finance_manager.clone(), acc.id());
                    }
                    view::account::Action::EditBookCheckingAccount(acc) => {
                        return self
                            .current_view
                            .book_checking_account_edit(self.finance_manager.clone(), acc.id());
                    }
                    view::account::Action::ViewTransaction(id) => {
                        return self
                            .current_view
                            .transaction(self.finance_manager.clone(), id);
                    }
                    view::account::Action::ViewAccount(id) => {
                        return self.current_view.account(self.finance_manager.clone(), id);
                    }
                    view::account::Action::AccountDeleted(acc_type) => match acc_type {
                        view::account::AccountType::AssetAccount => {
                            return self
                                .current_view
                                .asset_account_overview(self.finance_manager.clone());
                        }
                        view::account::AccountType::BookCheckingAccount => {
                            return self
                                .current_view
                                .book_checking_account_overview(self.finance_manager.clone());
                        }
                    },
                },
                ViewMessage::Transaction(m) => {
                    match message_match_action!(self, m, View::Transaction) {
                        view::transaction::Action::None => {}
                        view::transaction::Action::Edit(id) => {
                            return self
                                .current_view
                                .transaction_create(self.finance_manager.clone(), Some(id));
                        }
                        view::transaction::Action::ViewAccount(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id);
                        }
                        view::transaction::Action::Delete(task) => {
                            return task.map(|_| AppMessage::SwitchToFilterTransactionView);
                        }
                        view::transaction::Action::ViewBudget(id) => {
                            return self.current_view.budget(self.finance_manager.clone(), id);
                        }
                        view::transaction::Action::NewBillWithTransaction(transaction) => {
                            return self.current_view.new_bill_with_transaction(
                                self.finance_manager.clone(),
                                transaction,
                            );
                        }
                        view::transaction::Action::ViewCategory(category) => {
                            return self
                                .current_view
                                .category(self.finance_manager.clone(), category);
                        }
                    }
                }
                ViewMessage::Budget(m) => match message_match_action!(self, m, View::Budget) {
                    view::budget::Action::None => {}
                    view::budget::Action::ViewTransaction(id) => {
                        return self
                            .current_view
                            .transaction(self.finance_manager.clone(), id);
                    }
                    view::budget::Action::ViewAccount(id) => {
                        return self.current_view.account(self.finance_manager.clone(), id);
                    }
                    view::budget::Action::Edit(id) => {
                        return self
                            .current_view
                            .budget_edit(self.finance_manager.clone(), id);
                    }
                    view::budget::Action::Task(t) => {
                        return t.map(ViewMessage::Budget).map(AppMessage::ViewMessage);
                    }
                    view::budget::Action::DeletedBudget => {
                        return self
                            .current_view
                            .budget_overview(self.finance_manager.clone());
                    }
                },
                ViewMessage::CreateCategory(m) => {
                    match message_match_action!(self, m, View::CreateCategory) {
                        view::create_category::Action::CategoryCreated(id) => {
                            return self.current_view.category(self.finance_manager.clone(), id);
                        }
                        view::create_category::Action::None => {}
                        view::create_category::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateCategory)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_category::Action::Cancel => {
                            return self
                                .current_view
                                .category_overview(self.finance_manager.clone());
                        }
                        view::create_category::Action::CancelWithId(category_id) => {
                            return self
                                .current_view
                                .category(self.finance_manager.clone(), category_id);
                        }
                    }
                }
                ViewMessage::CategoryOverview(m) => {
                    match message_match_action!(self, m, View::CategoryOverview) {
                        view::category_overview::Action::ViewCategory(id) => {
                            return self.current_view.category(self.finance_manager.clone(), id);
                        }
                        view::category_overview::Action::NewCategory => {
                            self.current_view = View::CreateCategory(
                                view::create_category::CreateCategory::default(),
                            );
                        }
                        view::category_overview::Action::None => {}
                        view::category_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::CategoryOverview)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::Category(m) => match message_match_action!(self, m, View::Category) {
                    view::category::Action::Task(t) => {
                        return t.map(ViewMessage::Category).map(AppMessage::ViewMessage);
                    }
                    view::category::Action::None => {}
                    view::category::Action::EditCategory(id) => {
                        return self
                            .current_view
                            .category_edit(self.finance_manager.clone(), id);
                    }
                    view::category::Action::DeleteCategory(task) => {
                        return task.map(|_| AppMessage::SwitchToCategoryOverview);
                    }
                    view::category::Action::ViewTransaction(id) => {
                        return self
                            .current_view
                            .transaction(self.finance_manager.clone(), id);
                    }
                    view::category::Action::ViewAccount(id) => {
                        return self.current_view.account(self.finance_manager.clone(), id);
                    }
                },
                ViewMessage::BookCheckingAccountOverview(m) => {
                    match message_match_action!(self, m, View::BookCheckingAccountOverview) {
                        view::book_checking_account_overview::Action::ViewAccount(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id);
                        }
                        view::book_checking_account_overview::Action::None => {}
                        view::book_checking_account_overview::Action::CreateNewAccount => {
                            self.current_view = View::CreateBookCheckingAccount(
                                view::create_book_checking_account::CreateBookCheckingAccount::default(
                                ),
                            );
                        }
                        view::book_checking_account_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::BookCheckingAccountOverview)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::CreateBookCheckingAccount(m) => {
                    match message_match_action!(self, m, View::CreateBookCheckingAccount) {
                        view::create_book_checking_account::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateBookCheckingAccount)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_book_checking_account::Action::AccountCreated(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id);
                        }
                        view::create_book_checking_account::Action::None => {}
                        view::create_book_checking_account::Action::Cancel => {
                            return self
                                .current_view
                                .book_checking_account_overview(self.finance_manager.clone())
                        }
                        view::create_book_checking_account::Action::CancelWithId(acc_id) => {
                            return self
                                .current_view
                                .account(self.finance_manager.clone(), acc_id)
                        }
                    }
                }
                ViewMessage::Settings(m) => match message_match_action!(self, m, View::Settings) {
                    view::settings::Action::None => {}
                    view::settings::Action::ApplySettings(new_settings) => {
                        return self.apply_settings(new_settings);
                    }
                },
                ViewMessage::FilterTransaction(m) => {
                    match message_match_action!(self, m, View::FilterTransaction) {
                        view::filter_transactions::Action::None => {}
                        view::filter_transactions::Action::ViewTransaction(id) => {
                            return self
                                .current_view
                                .transaction(self.finance_manager.clone(), id);
                        }
                        view::filter_transactions::Action::ViewAccount(id) => {
                            return self.current_view.account(self.finance_manager.clone(), id);
                        }
                        view::filter_transactions::Action::Task(t) => {
                            return t
                                .map(ViewMessage::FilterTransaction)
                                .map(AppMessage::ViewMessage);
                        }
                    }
                }
                ViewMessage::CreateBill(m) => {
                    match message_match_action!(self, m, View::CreateBill) {
                        view::create_bill::Action::BillCreated(id) => {
                            return self.current_view.bill(self.finance_manager.clone(), id);
                        }
                        view::create_bill::Action::None => {}
                        view::create_bill::Action::Task(t) => {
                            return t.map(ViewMessage::CreateBill).map(AppMessage::ViewMessage);
                        }
                        view::create_bill::Action::Cancel => {
                            return self
                                .current_view
                                .bill_overview(self.finance_manager.clone())
                        }
                        view::create_bill::Action::CancelWithId(bill_id) => {
                            return self
                                .current_view
                                .bill(self.finance_manager.clone(), bill_id)
                        }
                    }
                }
                ViewMessage::BillOverview(m) => {
                    match message_match_action!(self, m, View::BillOverview) {
                        view::bill_overview::Action::ViewBill(id) => {
                            return self.current_view.bill(self.finance_manager.clone(), id);
                        }
                        view::bill_overview::Action::NewBill => {
                            return self
                                .current_view
                                .create_bill(self.finance_manager.clone(), None);
                        }
                        view::bill_overview::Action::None => {}

                        view::bill_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::BillOverview)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::Bill(m) => match message_match_action!(self, m, View::Bill) {
                    view::bill::Action::ViewTransaction(id) => {
                        return self
                            .current_view
                            .transaction(self.finance_manager.clone(), id);
                    }
                    view::bill::Action::Edit(id) => {
                        return self
                            .current_view
                            .create_bill(self.finance_manager.clone(), Some(id));
                    }
                    view::bill::Action::None => {}
                    view::bill::Action::Task(t) => {
                        return t.map(ViewMessage::Bill).map(AppMessage::ViewMessage);
                    }
                    view::bill::Action::Deleted => {
                        return self
                            .current_view
                            .bill_overview(self.finance_manager.clone());
                    }
                },
            },
            AppMessage::SwitchToBudgetOverview => {
                return self
                    .current_view
                    .budget_overview(self.finance_manager.clone());
            }

            AppMessage::SwitchToCreateTransActionView => {
                return self
                    .current_view
                    .transaction_create(self.finance_manager.clone(), None);
            }

            AppMessage::SwitchToAssetAccountsView => {
                return self
                    .current_view
                    .asset_account_overview(self.finance_manager.clone());
            }

            AppMessage::SwitchToCategoryOverview => {
                return self
                    .current_view
                    .category_overview(self.finance_manager.clone());
            }

            AppMessage::SwitchToBookCheckingAccountOverview => {
                return self
                    .current_view
                    .book_checking_account_overview(self.finance_manager.clone());
            }

            AppMessage::SwitchToSettingsView => {
                let (view, task) = view::settings::SettingsView::new(self.settings.clone());
                self.current_view = View::Settings(view);
                return task.map(ViewMessage::Settings).map(AppMessage::ViewMessage);
            }

            AppMessage::SwitchToFilterTransactionView => {
                self.current_view = View::Empty;
                return self
                    .current_view
                    .transaction_filter(self.finance_manager.clone());
            }
            AppMessage::SwitchToBillOverview => {
                return self
                    .current_view
                    .bill_overview(self.finance_manager.clone());
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<AppMessage> {
        fn icon_menu_item<'a>(
            text: &'a str,
            icon: &widget::svg::Handle,
            message: AppMessage,
            collapsed: bool,
        ) -> iced::Element<'a, AppMessage> {
            if collapsed {
                widget::tooltip(
                    widget::button(
                        widget::Svg::new(icon.clone())
                            .width(iced::Shrink)
                            .style(|theme: &iced::Theme, _| widget::svg::Style {
                                color: Some(theme.palette().primary),
                            })
                            .height(25),
                    )
                    .style(widget::button::text)
                    .on_press(message),
                    utils::style::container_popup_styling(widget::container(text)),
                    widget::tooltip::Position::Right,
                )
                .into()
            } else {
                widget::button(
                    utils::spal_row![
                        widget::Svg::new(icon.clone()).width(iced::Shrink).style(
                            |theme: &iced::Theme, _| widget::svg::Style {
                                color: Some(theme.palette().primary)
                            }
                        ),
                        text,
                    ]
                    .height(25),
                )
                .style(utils::style::button_sidebar)
                .on_press(message)
                .into()
            }
        }

        iced::widget::row![
            utils::spaced_column![
                widget::button(
                    widget::Svg::new(self.svg_cache.list.clone())
                        .style(|theme: &iced::Theme, _| widget::svg::Style {
                            color: Some(theme.palette().primary)
                        })
                        .width(iced::Shrink)
                )
                .on_press(AppMessage::ToggleSidebarCollapse)
                .style(widget::button::text),
                icon_menu_item(
                    "AssetAccounts",
                    &self.svg_cache.bank2,
                    AppMessage::SwitchToAssetAccountsView,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "BookCheckingAccounts",
                    &self.svg_cache.cash,
                    AppMessage::SwitchToBookCheckingAccountOverview,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "Budgets",
                    &self.svg_cache.piggy_bank_fill,
                    AppMessage::SwitchToBudgetOverview,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "Categories",
                    &self.svg_cache.bookmark_fill,
                    AppMessage::SwitchToCategoryOverview,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "Transactions",
                    &self.svg_cache.send_fill,
                    AppMessage::SwitchToFilterTransactionView,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "Bills",
                    &self.svg_cache.folder_fill,
                    AppMessage::SwitchToBillOverview,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "Create Transaction",
                    &self.svg_cache.plus_circle_fill,
                    AppMessage::SwitchToCreateTransActionView,
                    self.side_bar_collapsed
                ),
                widget::vertical_space(),
                icon_menu_item(
                    "Settings",
                    &self.svg_cache.gear_fill,
                    AppMessage::SwitchToSettingsView,
                    self.side_bar_collapsed
                ),
                icon_menu_item(
                    "License",
                    &self.svg_cache.book_fill,
                    AppMessage::SwitchToLicense,
                    self.side_bar_collapsed
                ),
            ]
            .align_x(iced::Alignment::Start),
            iced::widget::vertical_rule(5),
            iced::widget::container(
                match self.current_view {
                    View::Empty => widget::container(
                        widget::Svg::new(widget::svg::Handle::from_memory(include_bytes!(
                            "../../FM_Logo.svg"
                        )))
                        .width(iced::Fill)
                        .height(iced::Fill)
                    )
                    .padding(50)
                    .center(iced::Fill)
                    .into(),
                    View::Tutorial(ref items) => tutorial(items),
                    View::License =>
                        widget::scrollable(widget::text(include_str!("../../LICENSE"))).into(),
                    View::BudgetOverview(ref view) => view.view().map(ViewMessage::BudgetOverview),
                    View::CreateAssetAccount(ref view) =>
                        view.view().map(ViewMessage::CreateAssetAccount),
                    View::CreateBudget(ref view) => view.view().map(ViewMessage::CreateBudget),
                    View::CreateTransaction(ref view) =>
                        view.view().map(ViewMessage::CreateTransaction),
                    View::AssetAccounts(ref view) => view.view().map(ViewMessage::AssetAccounts),
                    View::Account(ref view) => view.view().map(ViewMessage::Account),
                    View::Transaction(ref view) => view.view().map(ViewMessage::Transaction),
                    View::Budget(ref view) => view.view().map(ViewMessage::Budget),
                    View::CreateCategory(ref view) => view.view().map(ViewMessage::CreateCategory),
                    View::CategoryOverview(ref view) =>
                        view.view().map(ViewMessage::CategoryOverview),
                    View::Category(ref view) => view.view().map(ViewMessage::Category),
                    View::BookCheckingAccountOverview(ref view) =>
                        view.view().map(ViewMessage::BookCheckingAccountOverview),
                    View::CreateBookCheckingAccount(ref view) =>
                        view.view().map(ViewMessage::CreateBookCheckingAccount),
                    View::Settings(ref view) => view.view().map(ViewMessage::Settings),
                    View::FilterTransaction(ref view) =>
                        view.view().map(ViewMessage::FilterTransaction),
                    View::CreateBill(ref view) => view.view().map(ViewMessage::CreateBill),
                    View::BillOverview(ref view) => view.view().map(ViewMessage::BillOverview),
                    View::Bill(ref view) => view.view().map(ViewMessage::Bill),
                }
                .map(AppMessage::ViewMessage)
            )
            .width(iced::Fill)
            .padding(utils::style::PADDING)
        ]
        .into()
    }

    fn apply_settings(&mut self, new_settings: settings::Settings) -> iced::Task<AppMessage> {
        let mut valid_settings = true;
        match new_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                if !matches!(
                    (*self.finance_manager).try_lock().unwrap().raw_fm(),
                    finance_managers::FinanceManagers::Ram(_)
                ) {
                    self.finance_manager =
                        Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                            finance_managers::FinanceManagers::Ram(
                                fm_core::managers::RamFinanceManager::default(),
                            ),
                        )));
                }
            }
            #[cfg(feature = "native")]
            settings::SelectedFinanceManager::SQLite => {
                let fm = match fm_core::managers::SqliteFinanceManager::new(
                    new_settings.finance_manager.sqlite_path.clone(),
                ) {
                    Ok(x) => Some(x),
                    Err(_) => {
                        if let View::Settings(settings_view) = &mut self.current_view {
                            settings_view.set_unsaved();
                        }
                        rfd::MessageDialog::new()
                            .set_title("Invalid SQLite Path")
                            .set_description("The provided SQLite path is invalid.")
                            .show();
                        valid_settings = false;
                        None
                    }
                };
                if let Some(manager) = fm {
                    self.finance_manager =
                        Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                            finance_managers::FinanceManagers::Sqlite(manager),
                        )));
                }
            }
            #[cfg(not(feature = "native"))]
            settings::SelectedFinanceManager::SQLite => {}
            settings::SelectedFinanceManager::Server => {
                self.finance_manager =
                    Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                        finance_managers::FinanceManagers::Server(
                            fm_server::client::Client::new((
                                new_settings.finance_manager.server_url.clone(),
                                new_settings.finance_manager.server_token.clone(),
                            ))
                            .unwrap(),
                        ),
                    )));
            }
        }
        if valid_settings {
            self.settings = new_settings.clone();
            let future = settings::write_settings(new_settings);
            iced::Task::future(async move {
                future.await.unwrap();
                AppMessage::Ignore
            })
        } else {
            iced::Task::none()
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    /// Verbose mode
    #[clap(short, long, default_value = "false")]
    verbose: bool,
    /// Debug mode
    #[clap(short, long, default_value = "false")]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // tracing / logging
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
    if args.verbose || args.debug {
        let stdout_log = tracing_subscriber::fmt::layer().compact();
        tracing_subscriber::registry()
            .with(stdout_log.with_filter(
                tracing_subscriber::filter::Targets::default().with_target(
                    "fm_gui",
                    if args.debug {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::INFO
                    },
                ),
            ))
            .init();
    }

    let loaded_settings = settings::read_settings().unwrap();

    let app = App::new(
        match loaded_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Ram(
                        fm_core::managers::RamFinanceManager::new(()).unwrap(),
                    ),
                )))
            }
            settings::SelectedFinanceManager::SQLite => {
                #[cfg(not(feature = "native"))]
                panic!("SQLite is not supported in the wasm version");
                #[cfg(feature = "native")]
                Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Sqlite(
                        if let Ok(fm) = fm_core::managers::SqliteFinanceManager::new(
                            loaded_settings.finance_manager.sqlite_path.clone(),
                        ) {
                            fm
                        } else {
                            rfd::MessageDialog::new()
                                .set_title("Invalid SQLite Path")
                                .set_description("The provided SQLite path is invalid.")
                                .show();
                            panic!("Invalid SQLite Path")
                        },
                    ),
                )))
            }
            settings::SelectedFinanceManager::Server => {
                Arc::new(Mutex::new(fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Server(
                        fm_server::client::Client::new((
                            loaded_settings.finance_manager.server_url.clone(),
                            loaded_settings.finance_manager.server_token.clone(),
                        ))
                        .unwrap(),
                    ),
                )))
            }
        },
        loaded_settings,
    );

    // run the gui
    iced::application("Finance Manager", App::update, App::view)
        .theme(|_| iced::Theme::Nord)
        .window(iced::window::Settings {
            icon: Some(
                iced::window::icon::from_file_data(include_bytes!("../assets/FM_Logo.png"), None)
                    .unwrap(),
            ),
            ..Default::default()
        })
        .run_with(|| (app, iced::Task::none()))
        .unwrap();
}

fn tutorial(items: &Vec<widget::markdown::Item>) -> iced::Element<ViewMessage> {
    widget::container(widget::scrollable(widget::column![
        utils::heading("Finance Manager", utils::HeadingLevel::H1),
        widget::markdown(
            items,
            utils::markdown_settings(),
            widget::markdown::Style::from_palette(iced::Theme::Nord.palette())
        )
        .map(|_| ViewMessage::None)
    ]))
    .center_x(iced::Fill)
    .into()
}
