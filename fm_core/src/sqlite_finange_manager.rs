use super::*;
use anyhow::{Context, Result};

#[derive(Clone)]
pub struct SqliteFinanceManager {
    path: String,
}

impl SqliteFinanceManager {
    pub fn new(path: String) -> Result<Self> {
        let new = Self { path };
        new.create_db()?;
        Ok(new)
    }

    fn create_db(&self) -> Result<()> {
        let connection = self.connect()?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS asset_account (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                notes TEXT,
                iban TEXT,
                bic TEXT
            );
            
            CREATE TABLE IF NOT EXISTS book_checking_account (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                notes TEXT,
                iban TEXT,
                bic TEXT
            );
            
            CREATE TABLE IF NOT EXISTS account (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                asset_account INTEGER,
                book_checking_account INTEGER,
                FOREIGN KEY(asset_account) REFERENCES asset_account(id),
                FOREIGN KEY (book_checking_account) REFERENCES book_checking_account(id)
            );
            
            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                amount_value INTEGER NOT NULL,
                currency INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                source INTEGER NOT NULL,
                destination INTEGER NOT NULL,
                budget INTEGER,
                date INTEGER,
                FOREIGN KEY(source) REFERENCES account(id),
                FOREIGN KEY(destination) REFERENCES account(id),
                FOREIGN KEY (budget) REFERENCES budget(id)
            );
            
            CREATE TABLE IF NOT EXISTS budget (
                id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                value INTEGER NOT NULL,
                currency INTEGER NOT NULL,
                timespan_type INTEGER NOT NULL,
                timespan_field1 INTEGER NOT NULL,
                timespan_field2 INTEGER
            );
        ",
        )?;
        Ok(())
    }

    fn connect(&self) -> Result<rusqlite::Connection> {
        Ok(rusqlite::Connection::open(&self.path).context("failed to open database")?)
    }
}

impl FinanceManager for SqliteFinanceManager {
    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::AssetAccount> {
        let connection = self.connect()?;
        todo!();
    }

    async fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::AssetAccount> {
        todo!()
    }

    async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        todo!()
    }

    async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        todo!()
    }

    async fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> Result<Currency> {
        todo!()
    }

    async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        todo!()
    }

    async fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        todo!()
    }

    async fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<Id>,
        date: DateTime,
    ) -> Result<Transaction> {
        todo!()
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::BookCheckingAccount> {
        todo!()
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> Result<Budget> {
        todo!()
    }

    async fn get_budgets(&self) -> Result<Vec<Budget>> {
        todo!()
    }

    async fn get_transactions_of_budget(&self, budget: &Budget) -> Result<Vec<Transaction>> {
        todo!()
    }
}
