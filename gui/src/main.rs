mod finance_managers;
mod settings;
mod sidebar;
mod view;

use clap::Parser;
use iced::widget;

use fm_core::FinanceManager;

macro_rules! message_match_action {
    ($view:expr, $v:path, $( $args:expr ),*) => {
        match $view {
            &mut $v(ref mut view) => view.update($( $args ),*),
            _ => {
                tracing::debug!("message not handled");
                return ViewAction::None;
            }
        }
    };
}

type Fm = fm_core::FMController<finance_managers::FinanceManagers>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
enum View {
    Markdown(String, Vec<widget::markdown::Item>),
    License,
    BudgetOverview(view::budget_overview::View),
    CreateAssetAccount(view::create_asset_account::View),
    CreateBudget(view::create_budget::View),
    CreateTransaction(view::create_transaction::View),
    AssetAccounts(view::asset_accounts_overview::View),
    Account(view::account::View),
    Transaction(view::transaction::View),
    Budget(view::budget::View),
    CreateCategory(view::create_category::View),
    CategoryOverview(view::category_overview::View),
    Category(view::category::View),
    BookCheckingAccountOverview(view::book_checking_account_overview::View),
    CreateBookCheckingAccount(view::create_book_checking_account::View),
    Settings(view::settings::View),
    FilterTransaction(view::filter_transactions::View),
    CreateBill(view::create_bill::View),
    BillOverview(view::bill_overview::View),
    Bill(view::bill::View),
}

impl std::fmt::Display for View {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Markdown(title, _) => write!(f, "{title}"),
            Self::License => write!(f, "License"),
            Self::BudgetOverview(_) => write!(f, "Budget Overview"),
            Self::CreateAssetAccount(_) => write!(f, "Create Asset Account"),
            Self::CreateBudget(_) => write!(f, "Create Budget"),
            Self::CreateTransaction(_) => write!(f, "Create Transaction"),
            Self::AssetAccounts(_) => write!(f, "Asset Account Overview"),
            Self::Account(_) => write!(f, "Account"),
            Self::Transaction(_) => write!(f, "Transaction"),
            Self::Budget(_) => write!(f, "Budget"),
            Self::CreateCategory(_) => write!(f, "Create Category"),
            Self::CategoryOverview(_) => write!(f, "Category Overview"),
            Self::Category(_) => write!(f, "Category"),
            Self::BookCheckingAccountOverview(_) => write!(f, "Book Checking Account Overview"),
            Self::CreateBookCheckingAccount(_) => write!(f, "Create Book Checking Account"),
            Self::Settings(_) => write!(f, "Settings"),
            Self::FilterTransaction(_) => write!(f, "Filter Transactions"),
            Self::CreateBill(_) => write!(f, "Create Bill"),
            Self::BillOverview(_) => write!(f, "Bill Overview"),
            Self::Bill(_) => write!(f, "Bill"),
        }
    }
}

impl View {
    fn account(&mut self, finance_controller: Fm, account: fm_core::Id) -> iced::Task<ViewMessage> {
        let (view, task) = view::account::View::fetch(finance_controller, account);
        *self = Self::Account(view);
        task.map(ViewMessage::Account)
    }

    fn asset_account_overview(&mut self, finance_controller: Fm) -> iced::Task<ViewMessage> {
        let (view, task) = view::asset_accounts_overview::View::fetch(finance_controller);
        *self = Self::AssetAccounts(view);
        task.map(ViewMessage::AssetAccounts)
    }

    fn bill_overview(&mut self, finance_controller: Fm) -> iced::Task<ViewMessage> {
        let (view, task) = view::bill_overview::View::fetch_unclosed(finance_controller);
        *self = Self::BillOverview(view);
        task.map(ViewMessage::BillOverview)
    }

    fn bill(&mut self, finance_controller: Fm, bill: fm_core::Id) -> iced::Task<ViewMessage> {
        let (view, task) = view::bill::View::fetch(bill, finance_controller);
        *self = Self::Bill(view);
        task.map(ViewMessage::Bill)
    }

    fn book_checking_account_overview(
        &mut self,
        finance_controller: Fm,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::book_checking_account_overview::View::fetch(finance_controller);
        *self = Self::BookCheckingAccountOverview(view);
        task.map(ViewMessage::BookCheckingAccountOverview)
    }

    fn budget_overview(
        &mut self,
        finance_controller: Fm,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::budget_overview::View::fetch(finance_controller, utc_offset);
        *self = Self::BudgetOverview(view);
        task.map(ViewMessage::BudgetOverview)
    }

    fn budget(
        &mut self,
        finance_controller: Fm,
        budget: fm_core::Id,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::budget::View::fetch(budget, 0, finance_controller, utc_offset);
        *self = Self::Budget(view);
        task.map(ViewMessage::Budget)
    }

    fn category_overview(&mut self, finance_controller: Fm) -> iced::Task<ViewMessage> {
        let (view, task) = view::category_overview::View::fetch(finance_controller);
        *self = View::CategoryOverview(view);
        task.map(ViewMessage::CategoryOverview)
    }

    fn category(
        &mut self,
        finance_controller: Fm,
        category: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::category::View::fetch(finance_controller, category);
        *self = Self::Category(view);
        task.map(ViewMessage::Category)
    }

    fn create_bill(
        &mut self,
        finance_controller: Fm,
        bill: Option<fm_core::Id>,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::create_bill::View::fetch(bill, finance_controller);
        *self = Self::CreateBill(view);
        task.map(ViewMessage::CreateBill)
    }

    fn budget_edit(
        &mut self,
        finance_controller: Fm,
        budget: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::create_budget::View::fetch(budget, finance_controller);
        *self = Self::CreateBudget(view);
        task.map(ViewMessage::CreateBudget)
    }

    fn transaction_create(
        &mut self,
        finance_controller: Fm,
        id: Option<fm_core::Id>,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = if let Some(id) = id {
            view::create_transaction::View::fetch(finance_controller, id)
        } else {
            view::create_transaction::View::new(finance_controller)
        };
        *self = Self::CreateTransaction(view);
        task.map(ViewMessage::CreateTransaction)
    }

    fn transaction_filter(&mut self, finance_controller: Fm) -> iced::Task<ViewMessage> {
        let (view, task) = view::filter_transactions::View::new(finance_controller);
        *self = Self::FilterTransaction(view);
        task.map(ViewMessage::FilterTransaction)
    }

    fn transaction(
        &mut self,
        finance_controller: Fm,
        transaction: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::transaction::View::fetch(transaction, finance_controller);
        *self = Self::Transaction(view);
        task.map(ViewMessage::Transaction)
    }

    fn category_edit(
        &mut self,
        finance_controller: Fm,
        category: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::create_category::View::fetch(category, finance_controller);
        *self = Self::CreateCategory(view);
        task.map(ViewMessage::CreateCategory)
    }

    fn asset_account_edit(
        &mut self,
        finance_controller: Fm,
        id: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::create_asset_account::View::fetch(id, finance_controller);
        *self = Self::CreateAssetAccount(view);
        task.map(ViewMessage::CreateAssetAccount)
    }

    fn book_checking_account_edit(
        &mut self,
        finance_controller: Fm,
        id: fm_core::Id,
    ) -> iced::Task<ViewMessage> {
        let (view, task) = view::create_book_checking_account::View::fetch(finance_controller, id);
        *self = Self::CreateBookCheckingAccount(view);
        task.map(ViewMessage::CreateBookCheckingAccount)
    }

    fn new_bill_with_transaction(
        &mut self,
        finance_controller: Fm,
        transaction: fm_core::Transaction,
    ) -> iced::Task<ViewMessage> {
        let (view, task) =
            view::create_bill::View::new_with_transaction(finance_controller, transaction);
        *self = Self::CreateBill(view);
        task.map(ViewMessage::CreateBill)
    }
}

fn view_update(
    finance_controller: Fm,
    utc_offset: time::UtcOffset,
    view: &mut View,
    message: ViewMessage,
) -> ViewAction {
    match message {
        ViewMessage::None => ViewAction::None,
        ViewMessage::BudgetOverview(m) => {
            match message_match_action!(view, View::BudgetOverview, m, finance_controller.clone()) {
                view::budget_overview::Action::None => ViewAction::None,
                view::budget_overview::Action::ViewBudget(id) => {
                    ViewAction::ViewTask(view.budget(finance_controller.clone(), id, utc_offset))
                }
                view::budget_overview::Action::CreateBudget => {
                    *view = View::CreateBudget(view::create_budget::View::default());
                    ViewAction::None
                }
                view::budget_overview::Action::Task(task) => {
                    ViewAction::ViewTask(task.map(ViewMessage::BudgetOverview))
                }
            }
        }
        ViewMessage::CreateAssetAccount(m) => {
            match message_match_action!(
                view,
                View::CreateAssetAccount,
                m,
                finance_controller.clone()
            ) {
                view::create_asset_account::Action::AssetAccountCreated(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::create_asset_account::Action::None => ViewAction::None,
                view::create_asset_account::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateAssetAccount))
                }
                view::create_asset_account::Action::Cancel => {
                    ViewAction::ViewTask(view.asset_account_overview(finance_controller.clone()))
                }
                view::create_asset_account::Action::CancelWithId(acc_id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), acc_id))
                }
            }
        }
        ViewMessage::CreateBudget(m) => {
            match message_match_action!(
                view,
                View::CreateBudget,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::create_budget::Action::BudgetCreated(id) => {
                    ViewAction::ViewTask(view.budget(finance_controller.clone(), id, utc_offset))
                }
                view::create_budget::Action::None => ViewAction::None,
                view::create_budget::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateBudget))
                }
                view::create_budget::Action::Cancel => ViewAction::ViewTask(
                    view.budget_overview(finance_controller.clone(), utc_offset),
                ),
                view::create_budget::Action::CancelWithId(budget_id) => ViewAction::ViewTask(
                    view.budget(finance_controller.clone(), budget_id, utc_offset),
                ),
            }
        }
        ViewMessage::CreateTransaction(m) => {
            match message_match_action!(
                view,
                View::CreateTransaction,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::create_transaction::Action::TransactionCreated(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::create_transaction::Action::None => ViewAction::None,
                view::create_transaction::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateTransaction))
                }
                view::create_transaction::Action::Cancel => {
                    ViewAction::ViewTask(view.transaction_filter(finance_controller.clone()))
                }
                view::create_transaction::Action::CancelWithId(transaction_id) => {
                    ViewAction::ViewTask(
                        view.transaction(finance_controller.clone(), transaction_id),
                    )
                }
            }
        }
        ViewMessage::AssetAccounts(m) => {
            match message_match_action!(view, View::AssetAccounts, m, finance_controller.clone()) {
                view::asset_accounts_overview::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::asset_accounts_overview::Action::CreateAssetAccount => {
                    *view = View::CreateAssetAccount(view::create_asset_account::View::default());
                    ViewAction::None
                }
                view::asset_accounts_overview::Action::None => ViewAction::None,
                view::asset_accounts_overview::Action::Task(task) => {
                    ViewAction::ViewTask(task.map(ViewMessage::AssetAccounts))
                }
            }
        }
        ViewMessage::Account(m) => {
            match message_match_action!(
                view,
                View::Account,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::account::Action::Task(t) => ViewAction::ViewTask(t.map(ViewMessage::Account)),
                view::account::Action::None => ViewAction::None,
                view::account::Action::EditAssetAccount(acc) => ViewAction::ViewTask(
                    view.asset_account_edit(finance_controller.clone(), acc.id),
                ),
                view::account::Action::EditBookCheckingAccount(acc) => ViewAction::ViewTask(
                    view.book_checking_account_edit(finance_controller.clone(), acc.id),
                ),
                view::account::Action::ViewTransaction(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::account::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::account::Action::AccountDeleted(acc_type) => match acc_type {
                    view::account::AccountType::AssetAccount => ViewAction::ViewTask(
                        view.asset_account_overview(finance_controller.clone()),
                    ),
                    view::account::AccountType::BookCheckingAccount => ViewAction::ViewTask(
                        view.book_checking_account_overview(finance_controller.clone()),
                    ),
                },
            }
        }
        ViewMessage::Transaction(m) => {
            match message_match_action!(view, View::Transaction, m, finance_controller.clone()) {
                view::transaction::Action::None => ViewAction::None,
                view::transaction::Action::Edit(id) => ViewAction::ViewTask(
                    view.transaction_create(finance_controller.clone(), Some(id)),
                ),
                view::transaction::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::transaction::Action::Delete(task) => {
                    ViewAction::AppTask(task.map(|_| AppMessage::SwitchToFilterTransactionView))
                }
                view::transaction::Action::ViewBudget(id) => {
                    ViewAction::ViewTask(view.budget(finance_controller.clone(), id, utc_offset))
                }
                view::transaction::Action::NewBillWithTransaction(transaction) => {
                    ViewAction::ViewTask(
                        view.new_bill_with_transaction(finance_controller.clone(), transaction),
                    )
                }
                view::transaction::Action::ViewCategory(category) => {
                    ViewAction::ViewTask(view.category(finance_controller.clone(), category))
                }
            }
        }
        ViewMessage::Budget(m) => {
            match message_match_action!(
                view,
                View::Budget,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::budget::Action::None => ViewAction::None,
                view::budget::Action::ViewTransaction(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::budget::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::budget::Action::Edit(id) => {
                    ViewAction::ViewTask(view.budget_edit(finance_controller.clone(), id))
                }
                view::budget::Action::Task(t) => ViewAction::ViewTask(t.map(ViewMessage::Budget)),
                view::budget::Action::DeletedBudget => ViewAction::ViewTask(
                    view.budget_overview(finance_controller.clone(), utc_offset),
                ),
            }
        }
        ViewMessage::CreateCategory(m) => {
            match message_match_action!(view, View::CreateCategory, m, finance_controller.clone()) {
                view::create_category::Action::CategoryCreated(id) => {
                    ViewAction::ViewTask(view.category(finance_controller.clone(), id))
                }
                view::create_category::Action::None => ViewAction::None,
                view::create_category::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateCategory))
                }
                view::create_category::Action::Cancel => {
                    ViewAction::ViewTask(view.category_overview(finance_controller.clone()))
                }
                view::create_category::Action::CancelWithId(category_id) => {
                    ViewAction::ViewTask(view.category(finance_controller.clone(), category_id))
                }
            }
        }
        ViewMessage::CategoryOverview(m) => {
            match message_match_action!(view, View::CategoryOverview, m, finance_controller.clone())
            {
                view::category_overview::Action::ViewCategory(id) => {
                    ViewAction::ViewTask(view.category(finance_controller.clone(), id))
                }
                view::category_overview::Action::NewCategory => {
                    *view = View::CreateCategory(view::create_category::View::default());
                    ViewAction::None
                }
                view::category_overview::Action::None => ViewAction::None,
                view::category_overview::Action::Task(task) => {
                    ViewAction::ViewTask(task.map(ViewMessage::CategoryOverview))
                }
            }
        }
        ViewMessage::Category(m) => {
            match message_match_action!(
                view,
                View::Category,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::category::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::Category))
                }
                view::category::Action::None => ViewAction::None,
                view::category::Action::EditCategory(id) => {
                    ViewAction::ViewTask(view.category_edit(finance_controller.clone(), id))
                }
                view::category::Action::DeleteCategory(task) => {
                    ViewAction::AppTask(task.map(|_| AppMessage::SwitchToCategoryOverview))
                }
                view::category::Action::ViewTransaction(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::category::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
            }
        }
        ViewMessage::BookCheckingAccountOverview(m) => {
            match message_match_action!(
                view,
                View::BookCheckingAccountOverview,
                m,
                finance_controller.clone()
            ) {
                view::book_checking_account_overview::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::book_checking_account_overview::Action::None => ViewAction::None,
                view::book_checking_account_overview::Action::CreateNewAccount => {
                    *view = View::CreateBookCheckingAccount(
                        view::create_book_checking_account::View::default(),
                    );
                    ViewAction::None
                }
                view::book_checking_account_overview::Action::Task(task) => {
                    ViewAction::ViewTask(task.map(ViewMessage::BookCheckingAccountOverview))
                }
            }
        }
        ViewMessage::CreateBookCheckingAccount(m) => {
            match message_match_action!(
                view,
                View::CreateBookCheckingAccount,
                m,
                finance_controller.clone()
            ) {
                view::create_book_checking_account::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateBookCheckingAccount))
                }
                view::create_book_checking_account::Action::AccountCreated(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::create_book_checking_account::Action::None => ViewAction::None,
                view::create_book_checking_account::Action::Cancel => ViewAction::ViewTask(
                    view.book_checking_account_overview(finance_controller.clone()),
                ),
                view::create_book_checking_account::Action::CancelWithId(acc_id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), acc_id))
                }
            }
        }
        ViewMessage::Settings(m) => {
            match message_match_action!(view, View::Settings, m, finance_controller) {
                view::settings::Action::None => ViewAction::None,
                view::settings::Action::ApplySettings(new_settings) => {
                    ViewAction::ApplySettings(new_settings)
                }
            }
        }
        ViewMessage::FilterTransaction(m) => {
            match message_match_action!(
                view,
                View::FilterTransaction,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::filter_transactions::Action::None => ViewAction::None,
                view::filter_transactions::Action::ViewTransaction(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::filter_transactions::Action::ViewAccount(id) => {
                    ViewAction::ViewTask(view.account(finance_controller.clone(), id))
                }
                view::filter_transactions::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::FilterTransaction))
                }
            }
        }
        ViewMessage::CreateBill(m) => {
            match message_match_action!(
                view,
                View::CreateBill,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                view::create_bill::Action::BillCreated(id) => {
                    ViewAction::ViewTask(view.bill(finance_controller.clone(), id))
                }
                view::create_bill::Action::None => ViewAction::None,
                view::create_bill::Action::Task(t) => {
                    ViewAction::ViewTask(t.map(ViewMessage::CreateBill))
                }
                view::create_bill::Action::Cancel => {
                    ViewAction::ViewTask(view.bill_overview(finance_controller.clone()))
                }
                view::create_bill::Action::CancelWithId(bill_id) => {
                    ViewAction::ViewTask(view.bill(finance_controller.clone(), bill_id))
                }
            }
        }
        ViewMessage::BillOverview(m) => {
            match message_match_action!(view, View::BillOverview, m, finance_controller.clone()) {
                view::bill_overview::Action::ViewBill(id) => {
                    ViewAction::ViewTask(view.bill(finance_controller.clone(), id))
                }
                view::bill_overview::Action::NewBill => {
                    ViewAction::ViewTask(view.create_bill(finance_controller.clone(), None))
                }
                view::bill_overview::Action::None => ViewAction::None,

                view::bill_overview::Action::Task(task) => {
                    ViewAction::ViewTask(task.map(ViewMessage::BillOverview))
                }
            }
        }
        ViewMessage::Bill(m) => {
            match message_match_action!(view, View::Bill, m, finance_controller.clone()) {
                view::bill::Action::ViewTransaction(id) => {
                    ViewAction::ViewTask(view.transaction(finance_controller.clone(), id))
                }
                view::bill::Action::Edit(id) => {
                    ViewAction::ViewTask(view.create_bill(finance_controller.clone(), Some(id)))
                }
                view::bill::Action::None => ViewAction::None,
                view::bill::Action::Task(t) => ViewAction::ViewTask(t.map(ViewMessage::Bill)),
                view::bill::Action::Deleted => {
                    ViewAction::ViewTask(view.bill_overview(finance_controller.clone()))
                }
            }
        }
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
    PaneViewMessage(widget::pane_grid::Pane, Box<ViewMessage>),
    PaneDragged(widget::pane_grid::DragEvent),
    PaneResize(widget::pane_grid::ResizeEvent),
    PaneSplit(widget::pane_grid::Axis, widget::pane_grid::Pane),
    PaneMaximize(widget::pane_grid::Pane),
    PaneClicked(widget::pane_grid::Pane),
    PaneClose(widget::pane_grid::Pane),
    PaneRestore,
    SideBarMessage(sidebar::Message),
    SwitchToFilterTransactionView,
    SwitchToCategoryOverview,
}

pub struct App {
    finance_controller: fm_core::FMController<finance_managers::FinanceManagers>,
    pane_grid: widget::pane_grid::State<View>,
    focused_pane: widget::pane_grid::Pane,
    side_bar: sidebar::Sidebar,
    settings: settings::Settings,
}

impl Default for App {
    fn default() -> Self {
        let finance_manager =
            fm_core::FMController::with_finance_manager(finance_managers::FinanceManagers::Ram(
                fm_core::managers::RamFinanceManager::new(()).unwrap(),
            ));
        let (pane_grid, focused_pane) = widget::pane_grid::State::new(View::Markdown(
            "Finance Manager".to_string(),
            widget::markdown::parse(include_str!("view/tutorial.md")).collect(),
        ));
        App {
            pane_grid,
            focused_pane,
            finance_controller: finance_manager,
            side_bar: sidebar::Sidebar::new(false),
            settings: settings::Settings::default(),
        }
    }
}

enum ViewAction {
    AppTask(iced::Task<AppMessage>),
    ViewTask(iced::Task<ViewMessage>),
    ApplySettings(settings::Settings),
    None,
}

impl App {
    fn new(finance_controller: Fm, settings: settings::Settings) -> Self {
        App {
            finance_controller,
            settings,
            ..Default::default()
        }
    }

    fn update(&mut self, message: AppMessage) -> iced::Task<AppMessage> {
        match message {
            AppMessage::Ignore => {}
            AppMessage::SwitchToCategoryOverview => {
                let pane = self.focused_pane;
                return self
                    .pane_grid
                    .get_mut(self.focused_pane)
                    .unwrap()
                    .category_overview(self.finance_controller.clone())
                    .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
            }
            AppMessage::SwitchToFilterTransactionView => {
                let pane = self.focused_pane;
                return self
                    .pane_grid
                    .get_mut(self.focused_pane)
                    .unwrap()
                    .transaction_filter(self.finance_controller.clone())
                    .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
            }
            AppMessage::SideBarMessage(m) => match self.side_bar.update(m) {
                sidebar::Action::None => {}
                sidebar::Action::SwitchToBudgetOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .budget_overview(
                            self.finance_controller.clone(),
                            time::UtcOffset::from_whole_seconds(self.settings.utc_seconds_offset)
                                .unwrap(),
                        )
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::CreateTransaction => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .transaction_create(self.finance_controller.clone(), None)
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::SwitchToAssetAccountView => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .asset_account_overview(self.finance_controller.clone())
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::SwitchToCategoryOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .category_overview(self.finance_controller.clone())
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::SwitchToBookCheckingAccountOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .book_checking_account_overview(self.finance_controller.clone())
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::SwitchToSettingsView => {
                    let (view, task) = view::settings::View::new(self.settings.clone());
                    *self.pane_grid.get_mut(self.focused_pane).unwrap() = View::Settings(view);
                    let pane = self.focused_pane;
                    return task
                        .map(ViewMessage::Settings)
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }

                sidebar::Action::SwitchToFilterTransactionView => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .transaction_filter(self.finance_controller.clone())
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToBillOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .bill_overview(self.finance_controller.clone())
                        .map(move |x| AppMessage::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToLicense => {
                    *self.pane_grid.get_mut(self.focused_pane).unwrap() = View::License;
                }
            },
            AppMessage::PaneClose(pane) => {
                if self.pane_grid.panes.len() > 1 {
                    self.pane_grid.close(pane);
                    if self.focused_pane == pane {
                        self.focused_pane = *self.pane_grid.panes.keys().next().unwrap();
                    }
                }
            }
            AppMessage::PaneDragged(event) => {
                if let widget::pane_grid::DragEvent::Dropped { pane, target } = event {
                    self.pane_grid.drop(pane, target);
                }
                self.focused_pane = *self.pane_grid.panes.keys().next().unwrap();
            }
            AppMessage::PaneResize(event) => {
                self.pane_grid.resize(event.split, event.ratio);
            }
            AppMessage::PaneSplit(axis, pane) => {
                self.pane_grid.split(
                    axis,
                    pane,
                    View::Markdown(
                        "Finance Manager".to_string(),
                        widget::markdown::parse(include_str!("view/tutorial.md")).collect(),
                    ),
                );
            }
            AppMessage::PaneMaximize(pane) => {
                self.pane_grid.maximize(pane);
            }
            AppMessage::PaneRestore => {
                self.pane_grid.restore();
            }
            AppMessage::PaneClicked(pane) => {
                self.focused_pane = pane;
            }

            AppMessage::PaneViewMessage(pane, view_message) => match self.pane_grid.get_mut(pane) {
                Some(current_view) => {
                    match view_update(
                        self.finance_controller.clone(),
                        time::UtcOffset::from_whole_seconds(self.settings.utc_seconds_offset)
                            .unwrap(),
                        current_view,
                        *view_message,
                    ) {
                        ViewAction::AppTask(task) => return task,
                        ViewAction::ViewTask(task) => {
                            return task.map(move |m| AppMessage::PaneViewMessage(pane, m.into()));
                        }
                        ViewAction::ApplySettings(new_settings) => {
                            return self.apply_settings(new_settings, Some(pane));
                        }
                        ViewAction::None => return iced::Task::none(),
                    }
                }
                None => return iced::Task::none(),
            },
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, AppMessage> {
        static PANE_BORDER_RADIUS: u16 = 5;

        iced::widget::row![
            self.side_bar.view().map(AppMessage::SideBarMessage),
            iced::widget::vertical_rule(5),
            iced::widget::container(
                widget::pane_grid::PaneGrid::new(
                    &self.pane_grid,
                    |pane, current_view, maximized| {
                        widget::pane_grid::Content::new(
                            widget::container(
                                match current_view {
                                    View::Markdown(_heading, items) => markdown(items),
                                    View::License => {
                                        widget::scrollable(include_str!("../../LICENSE"))
                                            .width(iced::Fill)
                                            .height(iced::Fill)
                                            .into()
                                    }
                                    View::BudgetOverview(view) => {
                                        view.view().map(ViewMessage::BudgetOverview)
                                    }
                                    View::CreateAssetAccount(view) => {
                                        view.view().map(ViewMessage::CreateAssetAccount)
                                    }
                                    View::CreateBudget(view) => {
                                        view.view().map(ViewMessage::CreateBudget)
                                    }
                                    View::CreateTransaction(view) => {
                                        view.view().map(ViewMessage::CreateTransaction)
                                    }
                                    View::AssetAccounts(view) => {
                                        view.view().map(ViewMessage::AssetAccounts)
                                    }
                                    View::Account(view) => view.view().map(ViewMessage::Account),
                                    View::Transaction(view) => {
                                        view.view().map(ViewMessage::Transaction)
                                    }
                                    View::Budget(view) => view.view().map(ViewMessage::Budget),
                                    View::CreateCategory(view) => {
                                        view.view().map(ViewMessage::CreateCategory)
                                    }
                                    View::CategoryOverview(view) => {
                                        view.view().map(ViewMessage::CategoryOverview)
                                    }
                                    View::Category(view) => view.view().map(ViewMessage::Category),
                                    View::BookCheckingAccountOverview(view) => {
                                        view.view().map(ViewMessage::BookCheckingAccountOverview)
                                    }
                                    View::CreateBookCheckingAccount(view) => {
                                        view.view().map(ViewMessage::CreateBookCheckingAccount)
                                    }
                                    View::Settings(view) => view.view().map(ViewMessage::Settings),
                                    View::FilterTransaction(view) => {
                                        view.view().map(ViewMessage::FilterTransaction)
                                    }
                                    View::CreateBill(view) => {
                                        view.view().map(ViewMessage::CreateBill)
                                    }
                                    View::BillOverview(view) => {
                                        view.view().map(ViewMessage::BillOverview)
                                    }
                                    View::Bill(view) => view.view().map(ViewMessage::Bill),
                                }
                                .map(move |m| AppMessage::PaneViewMessage(pane, m.into())),
                            )
                            .padding(style::PADDING)
                            .style(move |theme: &iced::Theme| {
                                let mut style =
                                    widget::container::background(theme.palette().background);
                                style.border.radius =
                                    style.border.radius.bottom(PANE_BORDER_RADIUS);
                                style.border.width = 5.0;
                                style.border.color = if pane == self.focused_pane {
                                    theme.extended_palette().primary.weak.color
                                } else {
                                    theme.extended_palette().secondary.strong.color
                                };
                                style
                            }),
                        )
                        .title_bar(
                            widget::pane_grid::TitleBar::new(widget::text(
                                current_view.to_string(),
                            ))
                            .controls(iced::Element::new(components::spaced_row![
                                pane_grid_control_buttons(icons::LAYOUT_SPLIT_HORIZONTAL.clone())
                                    .on_press(AppMessage::PaneSplit(
                                        widget::pane_grid::Axis::Vertical,
                                        pane
                                    )),
                                pane_grid_control_buttons(icons::LAYOUT_SPLIT_VERTICAL.clone())
                                    .on_press(AppMessage::PaneSplit(
                                        widget::pane_grid::Axis::Horizontal,
                                        pane
                                    )),
                                pane_grid_control_buttons(if maximized {
                                    icons::FULLSCREEN_EXIT.clone()
                                } else {
                                    icons::FULLSCREEN.clone()
                                })
                                .on_press(if maximized {
                                    AppMessage::PaneRestore
                                } else {
                                    AppMessage::PaneMaximize(pane)
                                }),
                                pane_grid_control_buttons(icons::X_LG.clone()).on_press_maybe(
                                    if self.pane_grid.panes.len() <= 1 {
                                        None
                                    } else {
                                        Some(AppMessage::PaneClose(pane))
                                    }
                                ),
                            ]))
                            .style(move |theme: &iced::Theme| {
                                let mut style =
                                    widget::container::background(if pane == self.focused_pane {
                                        theme.extended_palette().primary.weak.color
                                    } else {
                                        theme.extended_palette().secondary.strong.color
                                    });
                                style.border.radius = style.border.radius.top(PANE_BORDER_RADIUS);
                                style
                            })
                            .padding(style::PADDING),
                        )
                    }
                )
                .spacing(style::SPACING)
                .on_drag(AppMessage::PaneDragged)
                .on_resize(10, AppMessage::PaneResize)
                .on_click(AppMessage::PaneClicked)
            )
            .width(iced::Fill)
            .padding(style::PADDING)
        ]
        .into()
    }

    fn apply_settings(
        &mut self,
        new_settings: settings::Settings,
        pane: Option<widget::pane_grid::Pane>,
    ) -> iced::Task<AppMessage> {
        let mut valid_settings = true;
        match new_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                if !matches!(
                    *self.finance_controller.raw_fm().try_lock().unwrap(),
                    finance_managers::FinanceManagers::Ram(_)
                ) {
                    self.finance_controller = fm_core::FMController::with_finance_manager(
                        finance_managers::FinanceManagers::Ram(
                            fm_core::managers::RamFinanceManager::default(),
                        ),
                    );
                }
            }
            #[cfg(feature = "native")]
            settings::SelectedFinanceManager::SQLite => {
                let fm = match fm_core::managers::SqliteFinanceManager::new(
                    new_settings.finance_manager.sqlite_path.clone(),
                ) {
                    Ok(x) => Some(x),
                    Err(_) => {
                        if let Some(pane) = pane
                            && let View::Settings(settings_view) =
                                self.pane_grid.get_mut(pane).unwrap()
                        {
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
                    self.finance_controller = fm_core::FMController::with_finance_manager(
                        finance_managers::FinanceManagers::Sqlite(manager),
                    );
                }
            }
            #[cfg(not(feature = "native"))]
            settings::SelectedFinanceManager::SQLite => {}
            settings::SelectedFinanceManager::Server => {
                self.finance_controller = fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Server(
                        fm_server::client::Client::new((
                            new_settings.finance_manager.server_url.clone(),
                            new_settings.finance_manager.server_token.clone(),
                        ))
                        .unwrap(),
                    ),
                );
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
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
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

    let loaded_settings = match settings::read_settings() {
        Ok(loaded_setting) => loaded_setting,
        Err(err) => {
            let error_message = error::error_chain_string(err);
            error::blocking_error_popup(error_message.clone());
            panic!("{}", error_message);
        }
    };

    let app = App::new(
        match loaded_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                fm_core::FMController::with_finance_manager(finance_managers::FinanceManagers::Ram(
                    fm_core::managers::RamFinanceManager::new(()).unwrap(),
                ))
            }
            settings::SelectedFinanceManager::SQLite => {
                #[cfg(not(feature = "native"))]
                panic!("SQLite is not supported in the wasm version");
                #[cfg(feature = "native")]
                fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Sqlite(
                        match fm_core::managers::SqliteFinanceManager::new(
                            loaded_settings.finance_manager.sqlite_path.clone(),
                        ) {
                            Ok(fm) => fm,
                            Err(error) => {
                                rfd::MessageDialog::new()
                                    .set_title("Invalid SQLite Path")
                                    .set_description(error::error_chain_string(error))
                                    .show();
                                panic!("Invalid SQLite Path")
                            }
                        },
                    ),
                )
            }
            settings::SelectedFinanceManager::Server => {
                fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Server(
                        fm_server::client::Client::new((
                            loaded_settings.finance_manager.server_url.clone(),
                            loaded_settings.finance_manager.server_token.clone(),
                        ))
                        .unwrap(),
                    ),
                )
            }
        },
        loaded_settings,
    );

    // run the gui
    iced::application("Finance Manager", App::update, App::view)
        .theme(|_| iced::Theme::Nord)
        .window(iced::window::Settings {
            icon: Some(icons::FM_LOGO_WINDOW_ICON.clone()),
            ..Default::default()
        })
        .font(include_bytes!("../fonts/FiraSans-Regular.ttf"))
        .font(include_bytes!("../fonts/FiraSans-Bold.ttf"))
        .default_font(iced::Font::with_name("Fira Sans"))
        .run_with(|| (app, iced::Task::none()))
        .unwrap();
}

fn markdown(items: &Vec<widget::markdown::Item>) -> iced::Element<'_, ViewMessage> {
    widget::container(widget::scrollable(widget::column![
        widget::markdown(
            items,
            components::markdown_settings(),
            widget::markdown::Style::from_palette(iced::Theme::Nord.palette())
        )
        .map(|_| ViewMessage::None)
    ]))
    .center_x(iced::Fill)
    .width(iced::Fill)
    .height(iced::Fill)
    .into()
}

fn pane_grid_control_buttons(svg: widget::svg::Handle) -> widget::Button<'static, AppMessage> {
    widget::button(
        widget::svg(svg)
            .style(|theme: &iced::Theme, _| widget::svg::Style {
                color: Some(theme.palette().text),
            })
            .width(iced::Shrink),
    )
    .style(widget::button::secondary)
}
