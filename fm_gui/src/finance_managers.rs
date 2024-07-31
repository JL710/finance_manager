use anyhow::Result;

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

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: fm_core::Currency,
    ) -> Result<fm_core::account::AssetAccount> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .create_asset_account(name, note, iban, bic, offset)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .create_asset_account(name, note, iban, bic, offset)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.create_asset_account(name, note, iban, bic, offset)
                    .await
            }
        }
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .create_book_checking_account(name, note, iban, bic)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .create_book_checking_account(name, note, iban, bic)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.create_book_checking_account(name, note, iban, bic)
                    .await
            }
        }
    }

    async fn get_account_sum(
        &self,
        account: &fm_core::account::Account,
        date: fm_core::DateTime,
    ) -> Result<fm_core::Currency> {
        match self {
            FinanceManagers::Server(client) => client.get_account_sum(account, date).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_account_sum(account, date).await,
            FinanceManagers::Ram(ram) => ram.get_account_sum(account, date).await,
        }
    }

    async fn update_asset_account(
        &mut self,
        id: fm_core::Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: fm_core::Currency,
    ) -> Result<fm_core::account::AssetAccount> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .update_asset_account(id, name, note, iban, bic, offset)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .update_asset_account(id, name, note, iban, bic, offset)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.update_asset_account(id, name, note, iban, bic, offset)
                    .await
            }
        }
    }

    async fn update_book_checking_account(
        &mut self,
        id: fm_core::Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .update_book_checking_account(id, name, note, iban, bic)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .update_book_checking_account(id, name, note, iban, bic)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.update_book_checking_account(id, name, note, iban, bic)
                    .await
            }
        }
    }

    async fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: fm_core::Currency,
        transactions: Vec<(fm_core::Id, fm_core::Sign)>,
        due_date: Option<fm_core::DateTime>,
    ) -> Result<fm_core::Bill> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .create_bill(name, description, value, transactions, due_date)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .create_bill(name, description, value, transactions, due_date)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.create_bill(name, description, value, transactions, due_date)
                    .await
            }
        }
    }

    async fn update_bill(
        &mut self,
        id: fm_core::Id,
        name: String,
        description: Option<String>,
        value: fm_core::Currency,
        transactions: Vec<(fm_core::Id, fm_core::Sign)>,
        due_date: Option<fm_core::DateTime>,
    ) -> Result<()> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .update_bill(id, name, description, value, transactions, due_date)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .update_bill(id, name, description, value, transactions, due_date)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.update_bill(id, name, description, value, transactions, due_date)
                    .await
            }
        }
    }

    async fn get_bills(&self) -> Result<Vec<fm_core::Bill>> {
        match self {
            FinanceManagers::Server(client) => client.get_bills().await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_bills().await,
            FinanceManagers::Ram(ram) => ram.get_bills().await,
        }
    }

    async fn get_bill(&self, id: &fm_core::Id) -> Result<Option<fm_core::Bill>> {
        match self {
            FinanceManagers::Server(client) => client.get_bill(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_bill(id).await,
            FinanceManagers::Ram(ram) => ram.get_bill(id).await,
        }
    }

    async fn delete_bill(&mut self, id: fm_core::Id) -> Result<()> {
        match self {
            FinanceManagers::Server(client) => client.delete_bill(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.delete_bill(id).await,
            FinanceManagers::Ram(ram) => ram.delete_bill(id).await,
        }
    }

    async fn get_filtered_transactions(
        &self,
        filter: fm_core::transaction_filter::TransactionFilter,
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => client.get_filtered_transactions(filter).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_filtered_transactions(filter).await,
            FinanceManagers::Ram(ram) => ram.get_filtered_transactions(filter).await,
        }
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::Recouring,
    ) -> Result<fm_core::Budget> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .create_budget(name, description, total_value, timespan)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .create_budget(name, description, total_value, timespan)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.create_budget(name, description, total_value, timespan)
                    .await
            }
        }
    }

    async fn create_category(&mut self, name: String) -> Result<fm_core::Category> {
        match self {
            FinanceManagers::Server(client) => client.create_category(name).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.create_category(name).await,
            FinanceManagers::Ram(ram) => ram.create_category(name).await,
        }
    }

    async fn create_transaction(
        &mut self,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Or<fm_core::Id, String>, // id = Existing | String = New
        destination: fm_core::Or<fm_core::Id, String>,
        budget: Option<(fm_core::Id, fm_core::Sign)>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: Vec<(fm_core::Id, fm_core::Sign)>,
    ) -> Result<fm_core::Transaction> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .create_transaction(
                        amount,
                        title,
                        description,
                        source,
                        destination,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .create_transaction(
                        amount,
                        title,
                        description,
                        source,
                        destination,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.create_transaction(
                    amount,
                    title,
                    description,
                    source,
                    destination,
                    budget,
                    date,
                    metadata,
                    categories,
                )
                .await
            }
        }
    }

    async fn delete_category(&mut self, id: fm_core::Id) -> Result<()> {
        match self {
            FinanceManagers::Server(client) => client.delete_category(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.delete_category(id).await,
            FinanceManagers::Ram(ram) => ram.delete_category(id).await,
        }
    }

    async fn delete_transaction(&mut self, id: fm_core::Id) -> Result<()> {
        match self {
            FinanceManagers::Server(client) => client.delete_transaction(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.delete_transaction(id).await,
            FinanceManagers::Ram(ram) => ram.delete_transaction(id).await,
        }
    }

    async fn get_account(&self, id: fm_core::Id) -> Result<Option<fm_core::account::Account>> {
        match self {
            FinanceManagers::Server(client) => client.get_account(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_account(id).await,
            FinanceManagers::Ram(ram) => ram.get_account(id).await,
        }
    }

    async fn get_accounts(&self) -> Result<Vec<fm_core::account::Account>> {
        match self {
            FinanceManagers::Server(client) => client.get_accounts().await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_accounts().await,
            FinanceManagers::Ram(ram) => ram.get_accounts().await,
        }
    }

    async fn get_budget(&self, id: fm_core::Id) -> Result<Option<fm_core::Budget>> {
        match self {
            FinanceManagers::Server(client) => client.get_budget(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_budget(id).await,
            FinanceManagers::Ram(ram) => ram.get_budget(id).await,
        }
    }

    async fn get_budgets(&self) -> Result<Vec<fm_core::Budget>> {
        match self {
            FinanceManagers::Server(client) => client.get_budgets().await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_budgets().await,
            FinanceManagers::Ram(ram) => ram.get_budgets().await,
        }
    }

    async fn get_category(&self, id: fm_core::Id) -> Result<Option<fm_core::Category>> {
        match self {
            FinanceManagers::Server(client) => client.get_category(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_category(id).await,
            FinanceManagers::Ram(ram) => ram.get_category(id).await,
        }
    }

    async fn get_categories(&self) -> Result<Vec<fm_core::Category>> {
        match self {
            FinanceManagers::Server(client) => client.get_categories().await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_categories().await,
            FinanceManagers::Ram(ram) => ram.get_categories().await,
        }
    }

    async fn get_transaction(&self, id: fm_core::Id) -> Result<Option<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => client.get_transaction(id).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_transaction(id).await,
            FinanceManagers::Ram(ram) => ram.get_transaction(id).await,
        }
    }

    async fn get_transactions(
        &self,
        timespan: (Option<fm_core::DateTime>, Option<fm_core::DateTime>),
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => client.get_transactions(timespan).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_transactions(timespan).await,
            FinanceManagers::Ram(ram) => ram.get_transactions(timespan).await,
        }
    }

    async fn get_accounts_hash_map(
        &self,
    ) -> Result<std::collections::HashMap<fm_core::Id, fm_core::account::Account>> {
        match self {
            FinanceManagers::Server(client) => client.get_accounts_hash_map().await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_accounts_hash_map().await,
            FinanceManagers::Ram(ram) => ram.get_accounts_hash_map().await,
        }
    }

    async fn get_budget_transactions(
        &self,
        budget: &fm_core::Budget,
        offset: i32,
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => client.get_budget_transactions(budget, offset).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_budget_transactions(budget, offset).await,
            FinanceManagers::Ram(ram) => ram.get_budget_transactions(budget, offset).await,
        }
    }

    async fn get_budget_value(
        &self,
        budget: &fm_core::Budget,
        offset: i32,
    ) -> Result<fm_core::Currency> {
        match self {
            FinanceManagers::Server(client) => client.get_budget_value(budget, offset).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.get_budget_value(budget, offset).await,
            FinanceManagers::Ram(ram) => ram.get_budget_value(budget, offset).await,
        }
    }

    async fn get_relative_category_values(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<(fm_core::DateTime, fm_core::Currency)>> {
        match self {
            FinanceManagers::Server(client) => {
                client.get_relative_category_values(id, timespan).await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite.get_relative_category_values(id, timespan).await
            }
            FinanceManagers::Ram(ram) => ram.get_relative_category_values(id, timespan).await,
        }
    }

    async fn get_transactions_of_account(
        &self,
        account: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => {
                client.get_transactions_of_account(account, timespan).await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite.get_transactions_of_account(account, timespan).await
            }
            FinanceManagers::Ram(ram) => ram.get_transactions_of_account(account, timespan).await,
        }
    }

    async fn get_transactions_of_budget(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => {
                client.get_transactions_of_budget(id, timespan).await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite.get_transactions_of_budget(id, timespan).await
            }
            FinanceManagers::Ram(ram) => ram.get_transactions_of_budget(id, timespan).await,
        }
    }

    async fn get_transactions_of_category(
        &self,
        id: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        match self {
            FinanceManagers::Server(client) => {
                client.get_transactions_of_category(id, timespan).await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite.get_transactions_of_category(id, timespan).await
            }
            FinanceManagers::Ram(ram) => ram.get_transactions_of_category(id, timespan).await,
        }
    }

    async fn update_budget(
        &mut self,
        id: fm_core::Id,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::Recouring,
    ) -> Result<fm_core::Budget> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .update_budget(id, name, description, total_value, timespan)
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .update_budget(id, name, description, total_value, timespan)
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.update_budget(id, name, description, total_value, timespan)
                    .await
            }
        }
    }

    async fn update_category(
        &mut self,
        id: fm_core::Id,
        name: String,
    ) -> Result<fm_core::Category> {
        match self {
            FinanceManagers::Server(client) => client.update_category(id, name).await,
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => sqlite.update_category(id, name).await,
            FinanceManagers::Ram(ram) => ram.update_category(id, name).await,
        }
    }

    async fn update_transaction(
        &mut self,
        id: fm_core::Id,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Or<fm_core::Id, String>, // id = Existing | String = New
        destination: fm_core::Or<fm_core::Id, String>,
        budget: Option<(fm_core::Id, fm_core::Sign)>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: Vec<(fm_core::Id, fm_core::Sign)>,
    ) -> Result<fm_core::Transaction> {
        match self {
            FinanceManagers::Server(client) => {
                client
                    .update_transaction(
                        id,
                        amount,
                        title,
                        description,
                        source,
                        destination,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .await
            }
            #[cfg(feature = "native")]
            FinanceManagers::Sqlite(sqlite) => {
                sqlite
                    .update_transaction(
                        id,
                        amount,
                        title,
                        description,
                        source,
                        destination,
                        budget,
                        date,
                        metadata,
                        categories,
                    )
                    .await
            }
            FinanceManagers::Ram(ram) => {
                ram.update_transaction(
                    id,
                    amount,
                    title,
                    description,
                    source,
                    destination,
                    budget,
                    date,
                    metadata,
                    categories,
                )
                .await
            }
        }
    }
}
