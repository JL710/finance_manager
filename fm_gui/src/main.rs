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

#[derive(Debug)]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
enum View {
    Empty,
    Tutorial(Vec<widget::markdown::Item>),
    License,
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

#[derive(Debug, Clone)]
enum ViewMessage {
    None,
    BudgetOverview(view::budget_overview::Message),
    CreateAssetAccount(view::create_asset_account::Message),
    CreateBudget(view::create_budget::Message),
    CreateTransaction(view::create_transaction::MessageContainer),
    AssetAccounts(view::asset_accounts_overview::Message),
    ViewAccount(view::account::MessageContainer),
    Transaction(view::transaction::MessageContainer),
    ViewBudget(view::budget::MessageContainer),
    CreateCategory(view::create_category::Message),
    CategoryOverview(view::category_overview::Message),
    ViewCategory(view::category::Message),
    BookCheckingAccountOverview(view::book_checking_account_overview::Message),
    CreateBookCheckingAccount(view::create_book_checking_account::Message),
    Settings(view::settings::Message),
    FilterTransaction(view::filter_transactions::Message),
    CreateBill(view::create_bill::Message),
    BillOverview(view::bill_overview::Message),
    ViewBill(view::bill::MessageContainer),
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
    fn new(
        finance_manager: Arc<Mutex<fm_core::FMController<finance_managers::FinanceManagers>>>,
        settings: settings::Settings,
    ) -> Self {
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
                            return self.switch_view_budget(id);
                        }
                        view::budget_overview::Action::CreateBudget => {
                            self.current_view = View::CreateBudgetView(
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
                    match message_match_action!(self, m, View::CreateAssetAccountDialog) {
                        view::create_asset_account::Action::AssetAccountCreated(id) => {
                            return self.switch_view_account(id)
                        }
                        view::create_asset_account::Action::None => {}
                        view::create_asset_account::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateAssetAccount)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_asset_account::Action::Cancel => {
                            return self.switch_view_asset_account_overview()
                        }
                        view::create_asset_account::Action::CancelWithId(acc_id) => {
                            return self.switch_view_account(acc_id);
                        }
                    }
                }
                ViewMessage::CreateBudget(m) => {
                    match message_match_action!(self, m, View::CreateBudgetView) {
                        view::create_budget::Action::BudgetCreated(id) => {
                            return self.switch_view_budget(id);
                        }
                        view::create_budget::Action::None => {}
                        view::create_budget::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateBudget)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_budget::Action::Cancel => {
                            return self.switch_view_budget_overview();
                        }
                        view::create_budget::Action::CancelWithId(budget_id) => {
                            return self.switch_view_budget(budget_id);
                        }
                    }
                }
                ViewMessage::CreateTransaction(m) => {
                    match message_match_action!(self, m, View::CreateTransactionView) {
                        view::create_transaction::Action::TransactionCreated(id) => {
                            return self.switch_view_transaction(id);
                        }
                        view::create_transaction::Action::None => {}
                        view::create_transaction::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateTransaction)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_transaction::Action::Cancel => {
                            return self.switch_view_transaction_filter();
                        }
                        view::create_transaction::Action::CancelWithId(transaction_id) => {
                            return self.switch_view_transaction(transaction_id);
                        }
                    }
                }
                ViewMessage::AssetAccounts(m) => {
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
                        view::asset_accounts_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::AssetAccounts)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::ViewAccount(m) => {
                    match message_match_action!(self, m, View::ViewAccount) {
                        view::account::Action::Task(t) => {
                            return t.map(ViewMessage::ViewAccount).map(AppMessage::ViewMessage);
                        }
                        view::account::Action::None => {}
                        view::account::Action::EditAssetAccount(acc) => {
                            let (v, t) =
                                view::create_asset_account::CreateAssetAccountDialog::fetch(
                                    acc.id(),
                                    self.finance_manager.clone(),
                                );
                            self.current_view = View::CreateAssetAccountDialog(v);
                            return t
                                .map(ViewMessage::CreateAssetAccount)
                                .map(AppMessage::ViewMessage);
                        }
                        view::account::Action::EditBookCheckingAccount(acc) => {
                            let (v, t) =
                                view::create_book_checking_account::CreateBookCheckingAccount::fetch(
                                    self.finance_manager.clone(),
                                    acc.id(),
                                );
                            self.current_view = View::CreateBookCheckingAccount(v);
                            return t
                                .map(ViewMessage::CreateBookCheckingAccount)
                                .map(AppMessage::ViewMessage);
                        }
                        view::account::Action::ViewTransaction(id) => {
                            return self.switch_view_transaction(id);
                        }
                        view::account::Action::ViewAccount(id) => {
                            return self.switch_view_account(id);
                        }
                        view::account::Action::AccountDeleted(acc_type) => match acc_type {
                            view::account::AccountType::AssetAccount => {
                                return self.switch_view_asset_account_overview();
                            }
                            view::account::AccountType::BookCheckingAccount => {
                                return self.switch_view_book_checking_account_overview();
                            }
                        },
                    }
                }
                ViewMessage::Transaction(m) => {
                    match message_match_action!(self, m, View::TransactionView) {
                        view::transaction::Action::None => {}
                        view::transaction::Action::Edit(id) => {
                            let (v, t) = view::create_transaction::CreateTransactionView::fetch(
                                self.finance_manager.clone(),
                                id,
                            );
                            self.current_view = View::CreateTransactionView(v);
                            return t
                                .map(ViewMessage::CreateTransaction)
                                .map(AppMessage::ViewMessage);
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
                        view::transaction::Action::NewBillWithTransaction(transaction) => {
                            let (view, task) =
                                view::create_bill::CreateBillView::new_with_transaction(
                                    self.finance_manager.clone(),
                                    transaction,
                                );
                            self.current_view = View::CreateBill(view);
                            return task
                                .map(ViewMessage::CreateBill)
                                .map(AppMessage::ViewMessage);
                        }
                        view::transaction::Action::ViewCategory(category) => {
                            return self.switch_view_category(category);
                        }
                    }
                }
                ViewMessage::ViewBudget(m) => {
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
                            return t.map(ViewMessage::ViewBudget).map(AppMessage::ViewMessage);
                        }
                        view::budget::Action::DeletedBudget => {
                            return self.switch_view_budget_overview();
                        }
                    }
                }
                ViewMessage::CreateCategory(m) => {
                    match message_match_action!(self, m, View::CreateCategory) {
                        view::create_category::Action::CategoryCreated(id) => {
                            return self.switch_view_category(id);
                        }
                        view::create_category::Action::None => {}
                        view::create_category::Action::Task(t) => {
                            return t
                                .map(ViewMessage::CreateCategory)
                                .map(AppMessage::ViewMessage);
                        }
                        view::create_category::Action::Cancel => {
                            return self.switch_view_category_overview();
                        }
                        view::create_category::Action::CancelWithId(category_id) => {
                            return self.switch_view_category(category_id);
                        }
                    }
                }
                ViewMessage::CategoryOverview(m) => {
                    match message_match_action!(self, m, View::CategoryOverview) {
                        view::category_overview::Action::ViewCategory(id) => {
                            return self.switch_view_category(id);
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
                ViewMessage::ViewCategory(m) => {
                    match message_match_action!(self, m, View::ViewCategory) {
                        view::category::Action::Task(t) => {
                            return t
                                .map(ViewMessage::ViewCategory)
                                .map(AppMessage::ViewMessage);
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
                ViewMessage::BookCheckingAccountOverview(m) => {
                    match message_match_action!(self, m, View::BookCheckingAccountOverview) {
                        view::book_checking_account_overview::Action::ViewAccount(id) => {
                            return self.switch_view_account(id);
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
                            return self.switch_view_account(id);
                        }
                        view::create_book_checking_account::Action::None => {}
                        view::create_book_checking_account::Action::Cancel => {
                            return self.switch_view_book_checking_account_overview()
                        }
                        view::create_book_checking_account::Action::CancelWithId(acc_id) => {
                            return self.switch_view_account(acc_id)
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
                            return self.switch_view_transaction(id);
                        }
                        view::filter_transactions::Action::ViewAccount(id) => {
                            return self.switch_view_account(id);
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
                            return self.switch_view_bill(id);
                        }
                        view::create_bill::Action::None => {}
                        view::create_bill::Action::Task(t) => {
                            return t.map(ViewMessage::CreateBill).map(AppMessage::ViewMessage);
                        }
                        view::create_bill::Action::Cancel => {
                            return self.switch_view_bill_overview()
                        }
                        view::create_bill::Action::CancelWithId(bill_id) => {
                            return self.switch_view_bill(bill_id)
                        }
                    }
                }
                ViewMessage::BillOverview(m) => {
                    match message_match_action!(self, m, View::BillOverview) {
                        view::bill_overview::Action::ViewBill(id) => {
                            return self.switch_view_bill(id);
                        }
                        view::bill_overview::Action::NewBill => {
                            return self.switch_view_create_bill(None);
                        }
                        view::bill_overview::Action::None => {}

                        view::bill_overview::Action::Task(task) => {
                            return task
                                .map(ViewMessage::BillOverview)
                                .map(AppMessage::ViewMessage)
                        }
                    }
                }
                ViewMessage::ViewBill(m) => match message_match_action!(self, m, View::ViewBill) {
                    view::bill::Action::ViewTransaction(id) => {
                        return self.switch_view_transaction(id);
                    }
                    view::bill::Action::Edit(id) => {
                        return self.switch_view_create_bill(Some(id));
                    }
                    view::bill::Action::None => {}
                    view::bill::Action::Task(t) => {
                        return t.map(ViewMessage::ViewBill).map(AppMessage::ViewMessage);
                    }
                    view::bill::Action::Deleted => {
                        let (view, task) =
                            view::bill_overview::BillOverview::fetch(self.finance_manager.clone());
                        self.current_view = View::BillOverview(view);
                        return task
                            .map(ViewMessage::BillOverview)
                            .map(AppMessage::ViewMessage);
                    }
                },
            },
            AppMessage::SwitchToBudgetOverview => {
                return self.switch_view_budget_overview();
            }

            AppMessage::SwitchToCreateTransActionView => {
                return self.switch_view_transaction_create();
            }

            AppMessage::SwitchToAssetAccountsView => {
                return self.switch_view_asset_account_overview();
            }

            AppMessage::SwitchToCategoryOverview => {
                return self.switch_view_category_overview();
            }

            AppMessage::SwitchToBookCheckingAccountOverview => {
                return self.switch_view_book_checking_account_overview();
            }

            AppMessage::SwitchToSettingsView => {
                let (view, task) = view::settings::SettingsView::new(self.settings.clone());
                self.current_view = View::Settings(view);
                return task.map(ViewMessage::Settings).map(AppMessage::ViewMessage);
            }

            AppMessage::SwitchToFilterTransactionView => {
                self.current_view = View::Empty;
                return self.switch_view_transaction_filter();
            }
            AppMessage::SwitchToBillOverview => {
                return self.switch_view_bill_overview();
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
                    View::CreateAssetAccountDialog(ref view) =>
                        view.view().map(ViewMessage::CreateAssetAccount),
                    View::CreateBudgetView(ref view) => view.view().map(ViewMessage::CreateBudget),
                    View::CreateTransactionView(ref view) =>
                        view.view().map(ViewMessage::CreateTransaction),
                    View::AssetAccounts(ref view) => view.view().map(ViewMessage::AssetAccounts),
                    View::ViewAccount(ref view) => view.view().map(ViewMessage::ViewAccount),
                    View::TransactionView(ref view) => view.view().map(ViewMessage::Transaction),
                    View::ViewBudgetView(ref view) => view.view().map(ViewMessage::ViewBudget),
                    View::CreateCategory(ref view) => view.view().map(ViewMessage::CreateCategory),
                    View::CategoryOverview(ref view) =>
                        view.view().map(ViewMessage::CategoryOverview),
                    View::ViewCategory(ref view) => view.view().map(ViewMessage::ViewCategory),
                    View::BookCheckingAccountOverview(ref view) =>
                        view.view().map(ViewMessage::BookCheckingAccountOverview),
                    View::CreateBookCheckingAccount(ref view) =>
                        view.view().map(ViewMessage::CreateBookCheckingAccount),
                    View::Settings(ref view) => view.view().map(ViewMessage::Settings),
                    View::FilterTransaction(ref view) =>
                        view.view().map(ViewMessage::FilterTransaction),
                    View::CreateBill(ref view) => view.view().map(ViewMessage::CreateBill),
                    View::BillOverview(ref view) => view.view().map(ViewMessage::BillOverview),
                    View::ViewBill(ref view) => view.view().map(ViewMessage::ViewBill),
                }
                .map(AppMessage::ViewMessage)
            )
            .width(iced::Fill)
            .padding(utils::style::PADDING)
        ]
        .into()
    }

    fn switch_view_account(&mut self, account: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::account::Account::fetch(self.finance_manager.clone(), account);
        self.current_view = View::ViewAccount(view);
        task.map(ViewMessage::ViewAccount)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_asset_account_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::asset_accounts_overview::AssetAccountOverview::fetch(
            self.finance_manager.clone(),
        );
        self.current_view = View::AssetAccounts(view);
        task.map(ViewMessage::AssetAccounts)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_bill_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::bill_overview::BillOverview::fetch(self.finance_manager.clone());
        self.current_view = View::BillOverview(view);
        task.map(ViewMessage::BillOverview)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_bill(&mut self, bill: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::bill::Bill::fetch(bill, self.finance_manager.clone());
        self.current_view = View::ViewBill(view);
        task.map(ViewMessage::ViewBill).map(AppMessage::ViewMessage)
    }

    fn switch_view_book_checking_account_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) = view::book_checking_account_overview::BookCheckingAccountOverview::fetch(
            self.finance_manager.clone(),
        );
        self.current_view = View::BookCheckingAccountOverview(view);
        task.map(ViewMessage::BookCheckingAccountOverview)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_budget_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::budget_overview::BudgetOverview::fetch(self.finance_manager.clone());
        self.current_view = View::BudgetOverview(view);
        task.map(ViewMessage::BudgetOverview)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_budget(&mut self, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::budget::Budget::fetch(budget, 0, self.finance_manager.clone());
        self.current_view = View::ViewBudgetView(view);
        task.map(ViewMessage::ViewBudget)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_category_overview(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::category_overview::CategoryOverview::fetch(self.finance_manager.clone());
        self.current_view = View::CategoryOverview(view);
        task.map(ViewMessage::CategoryOverview)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_category(&mut self, category: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) = view::category::Category::fetch(self.finance_manager.clone(), category);
        self.current_view = View::ViewCategory(view);
        task.map(ViewMessage::ViewCategory)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_create_bill(&mut self, bill: Option<fm_core::Id>) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_bill::CreateBillView::fetch(bill, self.finance_manager.clone());
        self.current_view = View::CreateBill(view);
        task.map(ViewMessage::CreateBill)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_budget_edit(&mut self, budget: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_budget::CreateBudgetView::fetch(budget, self.finance_manager.clone());
        self.current_view = View::CreateBudgetView(view);
        task.map(ViewMessage::CreateBudget)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_transaction_create(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_transaction::CreateTransactionView::new(self.finance_manager.clone());
        self.current_view = View::CreateTransactionView(view);
        task.map(ViewMessage::CreateTransaction)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_transaction_filter(&mut self) -> iced::Task<AppMessage> {
        let (view, task) =
            view::filter_transactions::FilterTransactionView::new(self.finance_manager.clone());
        self.current_view = View::FilterTransaction(view);
        task.map(ViewMessage::FilterTransaction)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_transaction(&mut self, transaction: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::transaction::Transaction::fetch(transaction, self.finance_manager.clone());
        self.current_view = View::TransactionView(view);
        task.map(ViewMessage::Transaction)
            .map(AppMessage::ViewMessage)
    }

    fn switch_view_category_edit(&mut self, category: fm_core::Id) -> iced::Task<AppMessage> {
        let (view, task) =
            view::create_category::CreateCategory::fetch(category, self.finance_manager.clone());
        self.current_view = View::CreateCategory(view);
        task.map(ViewMessage::CreateCategory)
            .map(AppMessage::ViewMessage)
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
