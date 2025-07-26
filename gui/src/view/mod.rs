pub mod account;
pub mod asset_accounts_overview;
pub mod bill;
pub mod bill_overview;
pub mod book_checking_account_overview;
pub mod budget;
pub mod budget_overview;
pub mod category;
pub mod category_overview;
pub mod create_asset_account;
pub mod create_bill;
pub mod create_book_checking_account;
pub mod create_budget;
pub mod create_category;
pub mod create_transaction;
pub mod filter_transactions;
#[cfg(feature = "native")]
pub mod importer;
pub mod settings;
pub mod transaction;

use fm_core::{FMController, FinanceManager};
use iced::widget;

pub enum Action<FM: FinanceManager + 'static> {
    Task(iced::Task<Message<FM>>),
    ApplySettings(crate::settings::Settings),
    None,
}

#[derive(Debug, Clone)]
pub enum Message<FM: FinanceManager + 'static> {
    None,
    BudgetOverview(budget_overview::Message),
    CreateAssetAccount(create_asset_account::Message),
    CreateBudget(create_budget::Message),
    CreateTransaction(create_transaction::MessageContainer),
    AssetAccounts(asset_accounts_overview::Message),
    Account(account::MessageContainer),
    Transaction(transaction::MessageContainer),
    Budget(budget::MessageContainer),
    CreateCategory(create_category::Message),
    CategoryOverview(category_overview::Message),
    Category(category::Message),
    BookCheckingAccountOverview(book_checking_account_overview::Message),
    CreateBookCheckingAccount(create_book_checking_account::Message),
    Settings(settings::Message),
    FilterTransaction(filter_transactions::Message),
    CreateBill(create_bill::Message),
    BillOverview(bill_overview::Message),
    Bill(bill::MessageContainer),
    #[cfg(feature = "native")]
    Importer(importer::Message<FM>),
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
pub enum View<FM: FinanceManager + 'static> {
    Markdown(String, Vec<widget::markdown::Item>),
    License,
    BudgetOverview(budget_overview::View),
    CreateAssetAccount(create_asset_account::View),
    CreateBudget(create_budget::View),
    CreateTransaction(create_transaction::View),
    AssetAccounts(asset_accounts_overview::View),
    Account(account::View),
    Transaction(transaction::View),
    Budget(budget::View),
    CreateCategory(create_category::View),
    CategoryOverview(category_overview::View),
    Category(category::View),
    BookCheckingAccountOverview(book_checking_account_overview::View),
    CreateBookCheckingAccount(create_book_checking_account::View),
    Settings(settings::View),
    FilterTransaction(filter_transactions::View),
    CreateBill(create_bill::View),
    BillOverview(bill_overview::View),
    Bill(bill::View),
    #[cfg(feature = "native")]
    Importer(importer::Importer<FM>),
}

impl<FM: FinanceManager + 'static> std::fmt::Display for View<FM> {
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
            #[cfg(feature = "native")]
            Self::Importer(_) => write!(f, "Importer"),
        }
    }
}

impl<'a, FM: FinanceManager + 'static> From<&'a View<FM>> for iced::Element<'a, Message<FM>> {
    fn from(value: &'a View<FM>) -> Self {
        match value {
            View::Markdown(_heading, items) => markdown(items),
            View::License => widget::scrollable(include_str!("../../../LICENSE"))
                .width(iced::Fill)
                .height(iced::Fill)
                .into(),
            View::BudgetOverview(view) => view.view().map(Message::BudgetOverview),
            View::CreateAssetAccount(view) => view.view().map(Message::CreateAssetAccount),
            View::CreateBudget(view) => view.view().map(Message::CreateBudget),
            View::CreateTransaction(view) => view.view().map(Message::CreateTransaction),
            View::AssetAccounts(view) => view.view().map(Message::AssetAccounts),
            View::Account(view) => view.view().map(Message::Account),
            View::Transaction(view) => view.view().map(Message::Transaction),
            View::Budget(view) => view.view().map(Message::Budget),
            View::CreateCategory(view) => view.view().map(Message::CreateCategory),
            View::CategoryOverview(view) => view.view().map(Message::CategoryOverview),
            View::Category(view) => view.view().map(Message::Category),
            View::BookCheckingAccountOverview(view) => {
                view.view().map(Message::BookCheckingAccountOverview)
            }
            View::CreateBookCheckingAccount(view) => {
                view.view().map(Message::CreateBookCheckingAccount)
            }
            View::Settings(view) => view.view().map(Message::Settings),
            View::FilterTransaction(view) => view.view().map(Message::FilterTransaction),
            View::CreateBill(view) => view.view().map(Message::CreateBill),
            View::BillOverview(view) => view.view().map(Message::BillOverview),
            View::Bill(view) => view.view().map(Message::Bill),
            #[cfg(feature = "native")]
            View::Importer(view) => view.view().map(Message::Importer),
        }
    }
}

impl<FM: FinanceManager + 'static> View<FM> {
    pub fn account(
        &mut self,
        finance_controller: FMController<FM>,
        account: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = account::View::fetch(finance_controller, account);
        *self = Self::Account(view);
        task.map(Message::Account)
    }

    pub fn asset_account_overview(
        &mut self,
        finance_controller: FMController<FM>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = asset_accounts_overview::View::fetch(finance_controller);
        *self = Self::AssetAccounts(view);
        task.map(Message::AssetAccounts)
    }

    pub fn bill_overview(
        &mut self,
        finance_controller: FMController<FM>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = bill_overview::View::fetch_unclosed(finance_controller);
        *self = Self::BillOverview(view);
        task.map(Message::BillOverview)
    }

    pub fn bill(
        &mut self,
        finance_controller: FMController<FM>,
        bill: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = bill::View::fetch(bill, finance_controller);
        *self = Self::Bill(view);
        task.map(Message::Bill)
    }

    pub fn book_checking_account_overview(
        &mut self,
        finance_controller: FMController<FM>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = book_checking_account_overview::View::fetch(finance_controller);
        *self = Self::BookCheckingAccountOverview(view);
        task.map(Message::BookCheckingAccountOverview)
    }

    pub fn budget_overview(
        &mut self,
        finance_controller: FMController<FM>,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = budget_overview::View::fetch(finance_controller, utc_offset);
        *self = Self::BudgetOverview(view);
        task.map(Message::BudgetOverview)
    }

    pub fn budget(
        &mut self,
        finance_controller: FMController<FM>,
        budget: fm_core::Id,
        utc_offset: time::UtcOffset,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = budget::View::fetch(budget, 0, finance_controller, utc_offset);
        *self = Self::Budget(view);
        task.map(Message::Budget)
    }

    pub fn category_overview(
        &mut self,
        finance_controller: FMController<FM>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = category_overview::View::fetch(finance_controller);
        *self = View::CategoryOverview(view);
        task.map(Message::CategoryOverview)
    }

    pub fn category(
        &mut self,
        finance_controller: FMController<FM>,
        category: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = category::View::fetch(finance_controller, category);
        *self = Self::Category(view);
        task.map(Message::Category)
    }

    pub fn create_bill(
        &mut self,
        finance_controller: FMController<FM>,
        bill: Option<fm_core::Id>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_bill::View::fetch(bill, finance_controller);
        *self = Self::CreateBill(view);
        task.map(Message::CreateBill)
    }

    pub fn budget_edit(
        &mut self,
        finance_controller: FMController<FM>,
        budget: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_budget::View::fetch(budget, finance_controller);
        *self = Self::CreateBudget(view);
        task.map(Message::CreateBudget)
    }

    pub fn transaction_create(
        &mut self,
        finance_controller: FMController<FM>,
        id: Option<fm_core::Id>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = if let Some(id) = id {
            create_transaction::View::fetch(finance_controller, id)
        } else {
            create_transaction::View::new(finance_controller)
        };
        *self = Self::CreateTransaction(view);
        task.map(Message::CreateTransaction)
    }

    pub fn transaction_filter(
        &mut self,
        finance_controller: FMController<FM>,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = filter_transactions::View::new(finance_controller);
        *self = Self::FilterTransaction(view);
        task.map(Message::FilterTransaction)
    }

    pub fn transaction(
        &mut self,
        finance_controller: FMController<FM>,
        transaction: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = transaction::View::fetch(transaction, finance_controller);
        *self = Self::Transaction(view);
        task.map(Message::Transaction)
    }

    pub fn category_edit(
        &mut self,
        finance_controller: FMController<FM>,
        category: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_category::View::fetch(category, finance_controller);
        *self = Self::CreateCategory(view);
        task.map(Message::CreateCategory)
    }

    pub fn asset_account_edit(
        &mut self,
        finance_controller: FMController<FM>,
        id: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_asset_account::View::fetch(id, finance_controller);
        *self = Self::CreateAssetAccount(view);
        task.map(Message::CreateAssetAccount)
    }

    pub fn book_checking_account_edit(
        &mut self,
        finance_controller: FMController<FM>,
        id: fm_core::Id,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_book_checking_account::View::fetch(finance_controller, id);
        *self = Self::CreateBookCheckingAccount(view);
        task.map(Message::CreateBookCheckingAccount)
    }

    pub fn new_bill_with_transaction(
        &mut self,
        finance_controller: FMController<FM>,
        transaction: fm_core::Transaction,
    ) -> iced::Task<Message<FM>> {
        let (view, task) = create_bill::View::new_with_transaction(finance_controller, transaction);
        *self = Self::CreateBill(view);
        task.map(Message::CreateBill)
    }
}

macro_rules! message_match_action {
    ($view:expr, $v:path, $( $args:expr ),*) => {
        match $view {
            &mut $v(ref mut view) => view.update($( $args ),*),
            _ => {
                tracing::debug!("message not handled");
                return Action::None;
            }
        }
    };
}

pub fn view_update<FM: FinanceManager + 'static>(
    finance_controller: FMController<FM>,
    utc_offset: time::UtcOffset,
    view: &mut View<FM>,
    message: Message<FM>,
) -> Action<FM> {
    match message {
        Message::None => Action::None,
        Message::BudgetOverview(m) => {
            match message_match_action!(view, View::BudgetOverview, m, finance_controller.clone()) {
                budget_overview::Action::None => Action::None,
                budget_overview::Action::ViewBudget(id) => {
                    Action::Task(view.budget(finance_controller.clone(), id, utc_offset))
                }
                budget_overview::Action::CreateBudget => {
                    *view = View::CreateBudget(create_budget::View::default());
                    Action::None
                }
                budget_overview::Action::Task(task) => {
                    Action::Task(task.map(Message::BudgetOverview))
                }
            }
        }
        Message::CreateAssetAccount(m) => {
            match message_match_action!(
                view,
                View::CreateAssetAccount,
                m,
                finance_controller.clone()
            ) {
                create_asset_account::Action::AssetAccountCreated(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                create_asset_account::Action::None => Action::None,
                create_asset_account::Action::Task(t) => {
                    Action::Task(t.map(Message::CreateAssetAccount))
                }
                create_asset_account::Action::Cancel => {
                    Action::Task(view.asset_account_overview(finance_controller.clone()))
                }
                create_asset_account::Action::CancelWithId(acc_id) => {
                    Action::Task(view.account(finance_controller.clone(), acc_id))
                }
            }
        }
        Message::CreateBudget(m) => {
            match message_match_action!(
                view,
                View::CreateBudget,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                create_budget::Action::BudgetCreated(id) => {
                    Action::Task(view.budget(finance_controller.clone(), id, utc_offset))
                }
                create_budget::Action::None => Action::None,
                create_budget::Action::Task(t) => Action::Task(t.map(Message::CreateBudget)),
                create_budget::Action::Cancel => {
                    Action::Task(view.budget_overview(finance_controller.clone(), utc_offset))
                }
                create_budget::Action::CancelWithId(budget_id) => {
                    Action::Task(view.budget(finance_controller.clone(), budget_id, utc_offset))
                }
            }
        }
        Message::CreateTransaction(m) => {
            match message_match_action!(
                view,
                View::CreateTransaction,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                create_transaction::Action::TransactionCreated(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                create_transaction::Action::None => Action::None,
                create_transaction::Action::Task(t) => {
                    Action::Task(t.map(Message::CreateTransaction))
                }
                create_transaction::Action::Cancel => {
                    Action::Task(view.transaction_filter(finance_controller.clone()))
                }
                create_transaction::Action::CancelWithId(transaction_id) => {
                    Action::Task(view.transaction(finance_controller.clone(), transaction_id))
                }
            }
        }
        Message::AssetAccounts(m) => {
            match message_match_action!(view, View::AssetAccounts, m, finance_controller.clone()) {
                asset_accounts_overview::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                asset_accounts_overview::Action::CreateAssetAccount => {
                    *view = View::CreateAssetAccount(create_asset_account::View::default());
                    Action::None
                }
                asset_accounts_overview::Action::None => Action::None,
                asset_accounts_overview::Action::Task(task) => {
                    Action::Task(task.map(Message::AssetAccounts))
                }
            }
        }
        Message::Account(m) => {
            match message_match_action!(
                view,
                View::Account,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                account::Action::Task(t) => Action::Task(t.map(Message::Account)),
                account::Action::None => Action::None,
                account::Action::EditAssetAccount(acc) => {
                    Action::Task(view.asset_account_edit(finance_controller.clone(), acc.id))
                }
                account::Action::EditBookCheckingAccount(acc) => Action::Task(
                    view.book_checking_account_edit(finance_controller.clone(), acc.id),
                ),
                account::Action::ViewTransaction(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                account::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                account::Action::AccountDeleted(acc_type) => match acc_type {
                    account::AccountType::AssetAccount => {
                        Action::Task(view.asset_account_overview(finance_controller.clone()))
                    }
                    account::AccountType::BookCheckingAccount => Action::Task(
                        view.book_checking_account_overview(finance_controller.clone()),
                    ),
                },
            }
        }
        Message::Transaction(m) => {
            match message_match_action!(view, View::Transaction, m, finance_controller.clone()) {
                transaction::Action::None => Action::None,
                transaction::Action::Edit(id) => {
                    Action::Task(view.transaction_create(finance_controller.clone(), Some(id)))
                }
                transaction::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                transaction::Action::Delete(task) => Action::Task(
                    task.discard()
                        .chain(view.transaction_filter(finance_controller)),
                ),
                transaction::Action::ViewBudget(id) => {
                    Action::Task(view.budget(finance_controller.clone(), id, utc_offset))
                }
                transaction::Action::NewBillWithTransaction(transaction) => Action::Task(
                    view.new_bill_with_transaction(finance_controller.clone(), transaction),
                ),
                transaction::Action::ViewCategory(category) => {
                    Action::Task(view.category(finance_controller.clone(), category))
                }
            }
        }
        Message::Budget(m) => {
            match message_match_action!(
                view,
                View::Budget,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                budget::Action::None => Action::None,
                budget::Action::ViewTransaction(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                budget::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                budget::Action::Edit(id) => {
                    Action::Task(view.budget_edit(finance_controller.clone(), id))
                }
                budget::Action::Task(t) => Action::Task(t.map(Message::Budget)),
                budget::Action::DeletedBudget => {
                    Action::Task(view.budget_overview(finance_controller.clone(), utc_offset))
                }
            }
        }
        Message::CreateCategory(m) => {
            match message_match_action!(view, View::CreateCategory, m, finance_controller.clone()) {
                create_category::Action::CategoryCreated(id) => {
                    Action::Task(view.category(finance_controller.clone(), id))
                }
                create_category::Action::None => Action::None,
                create_category::Action::Task(t) => Action::Task(t.map(Message::CreateCategory)),
                create_category::Action::Cancel => {
                    Action::Task(view.category_overview(finance_controller.clone()))
                }
                create_category::Action::CancelWithId(category_id) => {
                    Action::Task(view.category(finance_controller.clone(), category_id))
                }
            }
        }
        Message::CategoryOverview(m) => {
            match message_match_action!(view, View::CategoryOverview, m, finance_controller.clone())
            {
                category_overview::Action::ViewCategory(id) => {
                    Action::Task(view.category(finance_controller.clone(), id))
                }
                category_overview::Action::NewCategory => {
                    *view = View::CreateCategory(create_category::View::default());
                    Action::None
                }
                category_overview::Action::None => Action::None,
                category_overview::Action::Task(task) => {
                    Action::Task(task.map(Message::CategoryOverview))
                }
            }
        }
        Message::Category(m) => {
            match message_match_action!(
                view,
                View::Category,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                category::Action::Task(t) => Action::Task(t.map(Message::Category)),
                category::Action::None => Action::None,
                category::Action::EditCategory(id) => {
                    Action::Task(view.category_edit(finance_controller.clone(), id))
                }
                category::Action::DeleteCategory(task) => Action::Task(
                    task.discard()
                        .chain(view.category_overview(finance_controller)),
                ),
                category::Action::ViewTransaction(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                category::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
            }
        }
        Message::BookCheckingAccountOverview(m) => {
            match message_match_action!(
                view,
                View::BookCheckingAccountOverview,
                m,
                finance_controller.clone()
            ) {
                book_checking_account_overview::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                book_checking_account_overview::Action::None => Action::None,
                book_checking_account_overview::Action::CreateNewAccount => {
                    *view = View::CreateBookCheckingAccount(
                        create_book_checking_account::View::default(),
                    );
                    Action::None
                }
                book_checking_account_overview::Action::Task(task) => {
                    Action::Task(task.map(Message::BookCheckingAccountOverview))
                }
            }
        }
        Message::CreateBookCheckingAccount(m) => {
            match message_match_action!(
                view,
                View::CreateBookCheckingAccount,
                m,
                finance_controller.clone()
            ) {
                create_book_checking_account::Action::Task(t) => {
                    Action::Task(t.map(Message::CreateBookCheckingAccount))
                }
                create_book_checking_account::Action::AccountCreated(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                create_book_checking_account::Action::None => Action::None,
                create_book_checking_account::Action::Cancel => {
                    Action::Task(view.book_checking_account_overview(finance_controller.clone()))
                }
                create_book_checking_account::Action::CancelWithId(acc_id) => {
                    Action::Task(view.account(finance_controller.clone(), acc_id))
                }
            }
        }
        Message::Settings(m) => {
            match message_match_action!(view, View::Settings, m, finance_controller) {
                settings::Action::None => Action::None,
                settings::Action::ApplySettings(new_settings) => {
                    Action::ApplySettings(new_settings)
                }
                settings::Action::Task(task) => Action::Task(task.map(Message::Settings)),
            }
        }
        Message::FilterTransaction(m) => {
            match message_match_action!(
                view,
                View::FilterTransaction,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                filter_transactions::Action::None => Action::None,
                filter_transactions::Action::ViewTransaction(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                filter_transactions::Action::ViewAccount(id) => {
                    Action::Task(view.account(finance_controller.clone(), id))
                }
                filter_transactions::Action::Task(t) => {
                    Action::Task(t.map(Message::FilterTransaction))
                }
            }
        }
        Message::CreateBill(m) => {
            match message_match_action!(
                view,
                View::CreateBill,
                m,
                finance_controller.clone(),
                utc_offset
            ) {
                create_bill::Action::BillCreated(id) => {
                    Action::Task(view.bill(finance_controller.clone(), id))
                }
                create_bill::Action::None => Action::None,
                create_bill::Action::Task(t) => Action::Task(t.map(Message::CreateBill)),
                create_bill::Action::Cancel => {
                    Action::Task(view.bill_overview(finance_controller.clone()))
                }
                create_bill::Action::CancelWithId(bill_id) => {
                    Action::Task(view.bill(finance_controller.clone(), bill_id))
                }
            }
        }
        Message::BillOverview(m) => {
            match message_match_action!(view, View::BillOverview, m, finance_controller.clone()) {
                bill_overview::Action::ViewBill(id) => {
                    Action::Task(view.bill(finance_controller.clone(), id))
                }
                bill_overview::Action::NewBill => {
                    Action::Task(view.create_bill(finance_controller.clone(), None))
                }
                bill_overview::Action::None => Action::None,

                bill_overview::Action::Task(task) => Action::Task(task.map(Message::BillOverview)),
            }
        }
        Message::Bill(m) => {
            match message_match_action!(view, View::Bill, m, finance_controller.clone()) {
                bill::Action::ViewTransaction(id) => {
                    Action::Task(view.transaction(finance_controller.clone(), id))
                }
                bill::Action::Edit(id) => {
                    Action::Task(view.create_bill(finance_controller.clone(), Some(id)))
                }
                bill::Action::None => Action::None,
                bill::Action::Task(t) => Action::Task(t.map(Message::Bill)),
                bill::Action::Deleted => {
                    Action::Task(view.bill_overview(finance_controller.clone()))
                }
            }
        }
        #[cfg(feature = "native")]
        Message::Importer(m) => {
            match message_match_action!(view, View::Importer, m, finance_controller.clone()) {
                importer::Action::None => Action::None,
                importer::Action::Task(task) => Action::Task(task.map(Message::Importer)),
            }
        }
    }
}

fn markdown<FM: FinanceManager + 'static>(
    items: &Vec<widget::markdown::Item>,
) -> iced::Element<'_, Message<FM>> {
    widget::container(widget::scrollable(widget::column![
        widget::markdown(
            items,
            components::markdown_settings(),
            widget::markdown::Style::from_palette(iced::Theme::Nord.palette())
        )
        .map(|_| Message::None)
    ]))
    .center_x(iced::Fill)
    .width(iced::Fill)
    .height(iced::Fill)
    .into()
}
