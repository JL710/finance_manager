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
                match message_match_action!(self, m, View::BudgetOverview) {
                    view::budget_overview::Action::None => {}
                    view::budget_overview::Action::ViewBudget(id) => {
                        return view::budget::switch_view_command(id, self.finance_manager.clone());
                    }
                    view::budget_overview::Action::CreateBudget => {
                        self.current_view = View::CreateBudgetView(
                            view::create_budget::CreateBudgetView::default(),
                        );
                    }
                }
            }
            AppMessage::SwitchToBudgetOverview => {
                return view::budget_overview::switch_view_command(self.finance_manager.clone())
            }
            AppMessage::CreateAssetAccountMessage(m) => {
                match message_match_action!(self, m, View::CreateAssetAccountDialog) {
                    view::create_asset_account::Action::CreateAssetAccount(t) => {
                        let manager = self.finance_manager.clone();
                        return t.then(move |id| {
                            let (v, task) = view::account::Account::fetch(manager.clone(), id);
                            iced::Task::done(AppMessage::SwitchView(View::ViewAccount(v)))
                                .chain(task.map(AppMessage::ViewAccountMessage))
                        });
                    }
                    view::create_asset_account::Action::None => {}
                }
            }
            AppMessage::CreateBudgetViewMessage(m) => {
                match message_match_action!(self, m, View::CreateBudgetView) {
                    view::create_budget::Action::CreateBudget(t) => {
                        let manager = self.finance_manager.clone();
                        return t.then(move |id| {
                            let (v, task) = view::budget::Budget::fetch(id, 0, manager.clone());
                            iced::Task::done(AppMessage::SwitchView(View::ViewBudgetView(v)))
                                .chain(task.map(AppMessage::ViewBudgetMessage))
                        });
                    }
                    view::create_budget::Action::None => {}
                }
            }
            AppMessage::CreateTransactionViewMessage(m) => {
                match message_match_action!(self, m, View::CreateTransactionView) {
                    view::create_transaction::Action::Task(t) => {
                        return t.map(AppMessage::CreateTransactionViewMessage);
                    }
                    view::create_transaction::Action::FinishedTransaction(task) => {
                        let manager = self.finance_manager.clone();
                        return task.then(move |transaction| {
                            view::transaction::switch_view_command(
                                *transaction.id(),
                                manager.clone(),
                            )
                        });
                    }
                    view::create_transaction::Action::None => {}
                }
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
                        return view::transaction::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::account::Action::ViewAccount(id) => {
                        return view::account::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                }
            }
            AppMessage::TransactionViewMessage(m) => {
                message_match!(self, m, View::TransactionView);
            }
            AppMessage::ViewBudgetMessage(m) => {
                match message_match_action!(self, m, View::ViewBudgetView) {
                    view::budget::Action::None => {}
                    view::budget::Action::ViewTransaction(id) => {
                        return view::transaction::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::budget::Action::ViewAccount(id) => {
                        return view::account::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::budget::Action::Edit(id) => {
                        return view::create_budget::switch_view_command_edit(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::budget::Action::Task(t) => {
                        return t.map(AppMessage::ViewBudgetMessage);
                    }
                }
            }
            AppMessage::CreateCategoryMessage(m) => {
                message_match!(self, m, View::CreateCategory);
            }
            AppMessage::CategoryOverviewMessage(m) => {
                match message_match_action!(self, m, View::CategoryOverview) {
                    view::category_overview::Action::ViewCategory(id) => {
                        return view::category::switch_view_command(
                            self.finance_manager.clone(),
                            id,
                        );
                    }
                    view::category_overview::Action::NewCategory => {
                        self.current_view =
                            View::CreateCategory(view::create_category::CreateCategory::default());
                    }
                    view::category_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToCategoryOverview => {
                return view::category_overview::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::ViewCategoryMessage(m) => {
                match message_match_action!(self, m, View::ViewCategory) {
                    view::category::Action::Task(t) => {
                        return t.map(AppMessage::ViewCategoryMessage);
                    }
                    view::category::Action::None => {}
                    view::category::Action::EditCategory(id) => {
                        let manager = self.finance_manager.clone();
                        return iced::Task::future(async move {
                            use fm_core::FinanceManager;
                            let category = manager
                                .lock()
                                .await
                                .get_category(id)
                                .await
                                .expect("Failed to get category")
                                .unwrap();
                            AppMessage::SwitchView(View::CreateCategory(
                                view::create_category::CreateCategory::new(
                                    Some(*category.id()),
                                    category.name().to_owned(),
                                ),
                            ))
                        });
                    }
                    view::category::Action::DeleteCategory(task) => {
                        return task.map(|_| AppMessage::SwitchToCategoryOverview);
                    }
                    view::category::Action::ViewTransaction(id) => {
                        return view::transaction::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::category::Action::ViewAccount(id) => {
                        return view::account::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                }
            }
            AppMessage::BookCheckingAccountOverviewMessage(m) => {
                match message_match_action!(self, m, View::BookCheckingAccountOverview) {
                    view::book_checking_account_overview::Action::ViewAccount(id) => {
                        return view::account::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::book_checking_account_overview::Action::None => {}
                }
            }
            AppMessage::SwitchToBookCheckingAccountOverview => {
                return view::book_checking_account_overview::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::CreateBookCheckingAccountMessage(m) => {
                match message_match_action!(self, m, View::CreateBookCheckingAccount) {
                    view::create_book_checking_account::Action::CreateAccount(t) => {
                        let manager = self.finance_manager.clone();
                        return t.then(move |id| {
                            let (v, task) = view::account::Account::fetch(manager.clone(), id);
                            iced::Task::done(AppMessage::SwitchView(View::ViewAccount(v)))
                                .chain(task.map(AppMessage::ViewAccountMessage))
                        });
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
                message_match!(self, m, View::FilterTransaction);
            }
            AppMessage::CreateBillMessage(m) => {
                match message_match_action!(self, m, View::CreateBill) {
                    view::create_bill::Action::CreateBill(t) => {
                        let manager = self.finance_manager.clone();
                        return t.then(move |id| {
                            let (v, task) = view::bill::Bill::fetch(id, manager.clone());
                            iced::Task::done(AppMessage::SwitchView(View::ViewBill(v)))
                                .chain(task.map(AppMessage::ViewBillMessage))
                        });
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
                        return view::bill::switch_view_command(id, self.finance_manager.clone());
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
                return view::filter_transactions::switch_view_command(
                    self.finance_manager.clone(),
                );
            }
            AppMessage::SwitchToBillOverview => {
                return view::bill_overview::switch_view_command(self.finance_manager.clone());
            }
            AppMessage::ViewBillMessage(m) => {
                match message_match_action!(self, m, View::ViewBill) {
                    view::bill::Action::ViewTransaction(id) => {
                        return view::transaction::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::bill::Action::Edit(id) => {
                        return view::create_bill::switch_view_command(
                            id,
                            self.finance_manager.clone(),
                        );
                    }
                    view::bill::Action::None => {}
                }
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
