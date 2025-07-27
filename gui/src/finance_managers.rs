use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone)]
pub enum FinanceManagers {
    Server(fm_server::client::Client),
    #[cfg(feature = "native")]
    Sqlite(fm_core::managers::SqliteFinanceManager),
    Ram(fm_core::managers::RamFinanceManager),
}

impl std::fmt::Debug for FinanceManagers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinanceManagers::Server(_) => write!(f, "Server"),
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(_) => write!(f, "Sqlite"),
            FinanceManagers::Ram(_) => write!(f, "Ram"),
        }
    }
}

impl std::fmt::Display for FinanceManagers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinanceManagers::Server(fm) => write!(f, "Server {}", fm.url()),
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(fm) => write!(f, "Sqlite {}", fm.path()),
            FinanceManagers::Ram(_) => write!(f, "Ram"),
        }
    }
}

impl Default for FinanceManagers {
    fn default() -> Self {
        FinanceManagers::Ram(fm_core::managers::ram_finance_manager::RamFinanceManager::default())
    }
}

macro_rules! fm_match {
    ($self_fm: expr, $func_name: ident, $( $args:expr ),*) => {
        match $self_fm {
            FinanceManagers::Server(client) => {
                client
                    .$func_name($( $args ),*)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .$func_name($( $args ),*)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.$func_name($( $args ),*)
                    .await
            }
        }
    }
}

impl fm_core::FinanceManager for FinanceManagers {
    #[cfg(feature = "native")]
    type Flags = (
        Option<fm_server::client::Client>,
        Option<fm_core::managers::RamFinanceManager>,
        Option<fm_core::managers::SqliteFinanceManager>,
    );
    #[cfg(not(feature = "native"))]
    type Flags = (
        Option<fm_server::client::Client>,
        Option<fm_core::managers::RamFinanceManager>,
        Option<()>,
    );

    fn new(flags: Self::Flags) -> Result<Self> {
        match flags {
            (Some(client), None, None) => Ok(FinanceManagers::Server(client)),
            (None, Some(ram), None) => Ok(FinanceManagers::Ram(ram)),
            #[cfg(feature = "native")]
            (None, None, Some(sqlite)) => Ok(FinanceManagers::Sqlite(sqlite)),
            _ => Err(anyhow::anyhow!("Invalid flags")),
        }
    }

    async fn last_modified(&self) -> Result<fm_core::DateTime> {
        fm_match!(self, last_modified,)
    }

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<fm_core::AccountId>,
        bic: Option<fm_core::Bic>,
        offset: fm_core::Currency,
    ) -> Result<fm_core::account::AssetAccount> {
        fm_match!(self, create_asset_account, name, note, iban, bic, offset)
    }

    async fn delete_account(&mut self, id: fm_core::Id) -> Result<()> {
        fm_match!(self, delete_account, id)
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<fm_core::AccountId>,
        bic: Option<fm_core::Bic>,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        fm_match!(self, create_book_checking_account, name, note, iban, bic)
    }

    async fn get_account_sum(
        &self,
        account: &fm_core::account::Account,
        date: fm_core::DateTime,
    ) -> Result<fm_core::Currency> {
        fm_match!(self, get_account_sum, account, date)
    }

    async fn update_asset_account(
        &mut self,
        account: fm_core::account::AssetAccount,
    ) -> Result<fm_core::account::AssetAccount> {
        fm_match!(self, update_asset_account, account)
    }

    async fn update_book_checking_account(
        &mut self,
        account: fm_core::account::BookCheckingAccount,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        fm_match!(self, update_book_checking_account, account)
    }

    async fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: fm_core::Currency,
        transactions: HashMap<fm_core::Id, fm_core::Sign>,
        due_date: Option<fm_core::DateTime>,
        closed: bool,
    ) -> Result<fm_core::Bill> {
        fm_match!(
            self,
            create_bill,
            name,
            description,
            value,
            transactions,
            due_date,
            closed
        )
    }

    async fn update_bill(&mut self, bill: fm_core::Bill) -> Result<()> {
        fm_match!(self, update_bill, bill)
    }

    async fn get_bills(&self, closed: Option<bool>) -> Result<Vec<fm_core::Bill>> {
        fm_match!(self, get_bills, closed)
    }

    async fn get_bill(&self, id: &fm_core::Id) -> Result<Option<fm_core::Bill>> {
        fm_match!(self, get_bill, id)
    }

    async fn delete_bill(&mut self, id: fm_core::Id) -> Result<()> {
        fm_match!(self, delete_bill, id)
    }

    async fn get_filtered_transactions(
        &self,
        filter: fm_core::transaction_filter::TransactionFilter,
    ) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_filtered_transactions, filter)
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::budget::Recurring,
    ) -> Result<fm_core::Budget> {
        fm_match!(
            self,
            create_budget,
            name,
            description,
            total_value,
            timespan
        )
    }

    async fn delete_budget(&mut self, id: fm_core::Id) -> Result<()> {
        fm_match!(self, delete_budget, id)
    }

    async fn create_category(&mut self, name: String) -> Result<fm_core::Category> {
        fm_match!(self, create_category, name)
    }

    async fn create_transaction(
        &mut self,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Id,
        destination: fm_core::Id,
        budget: Option<(fm_core::Id, fm_core::Sign)>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: HashMap<fm_core::Id, fm_core::Sign>,
    ) -> Result<fm_core::Transaction> {
        fm_match!(
            self,
            create_transaction,
            amount,
            title,
            description,
            source,
            destination,
            budget,
            date,
            metadata,
            categories
        )
    }

    async fn delete_category(&mut self, id: fm_core::Id) -> Result<()> {
        fm_match!(self, delete_category, id)
    }

    async fn delete_transaction(&mut self, id: fm_core::Id) -> Result<()> {
        fm_match!(self, delete_transaction, id)
    }

    async fn get_account(&self, id: fm_core::Id) -> Result<Option<fm_core::account::Account>> {
        fm_match!(self, get_account, id)
    }

    async fn get_accounts(&self) -> Result<Vec<fm_core::account::Account>> {
        fm_match!(self, get_accounts,)
    }

    async fn get_budget(&self, id: fm_core::Id) -> Result<Option<fm_core::Budget>> {
        fm_match!(self, get_budget, id)
    }

    async fn get_budgets(&self) -> Result<Vec<fm_core::Budget>> {
        fm_match!(self, get_budgets,)
    }

    async fn get_category(&self, id: fm_core::Id) -> Result<Option<fm_core::Category>> {
        fm_match!(self, get_category, id)
    }

    async fn get_categories(&self) -> Result<Vec<fm_core::Category>> {
        fm_match!(self, get_categories,)
    }

    async fn get_transaction(&self, id: fm_core::Id) -> Result<Option<fm_core::Transaction>> {
        fm_match!(self, get_transaction, id)
    }

    async fn get_transactions_in_timespan(
        &self,
        timespan: (Option<fm_core::DateTime>, Option<fm_core::DateTime>),
    ) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_transactions_in_timespan, timespan)
    }

    async fn get_transactions(&self, ids: Vec<fm_core::Id>) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_transactions, ids)
    }

    async fn get_accounts_hash_map(
        &self,
    ) -> Result<std::collections::HashMap<fm_core::Id, fm_core::account::Account>> {
        fm_match!(self, get_accounts_hash_map,)
    }

    async fn get_relative_category_values(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<(fm_core::DateTime, fm_core::Currency)>> {
        fm_match!(self, get_relative_category_values, id, timespan)
    }

    async fn get_transactions_of_account(
        &self,
        account: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_transactions_of_account, account, timespan)
    }

    async fn get_transactions_of_budget(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_transactions_of_budget, id, timespan)
    }

    async fn get_transactions_of_category(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        fm_match!(self, get_transactions_of_category, id, timespan)
    }

    async fn update_budget(&mut self, budget: fm_core::Budget) -> Result<fm_core::Budget> {
        fm_match!(self, update_budget, budget)
    }

    async fn update_category(&mut self, category: fm_core::Category) -> Result<fm_core::Category> {
        fm_match!(self, update_category, category)
    }

    async fn update_transaction(
        &mut self,
        transaction: fm_core::Transaction,
    ) -> Result<fm_core::Transaction> {
        fm_match!(self, update_transaction, transaction)
    }
}
