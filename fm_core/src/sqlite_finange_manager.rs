use crate::account::AssetAccount;

use super::*;
use anyhow::{Context, Result};

type TransactionSignature = (
    Id,
    f64,
    i32,
    String,
    Option<String>,
    Id,
    Id,
    Option<Id>,
    Option<bool>,
    i64,
    String,
);

const TRANSACTION_FIELDS: &str = "id, amount_value, currency, title, description, source_id, destination_id, budget, budget_sign, timestamp, metadata";

impl TryInto<Transaction> for TransactionSignature {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Transaction> {
        Ok(Transaction::new(
            self.0,
            Currency::from_currency_id(self.2, self.1)?,
            self.3,
            self.4,
            self.5,
            self.6,
            self.7.map(|x| {
                (
                    x,
                    if self.8.unwrap() {
                        Sign::Positive
                    } else {
                        Sign::Negative
                    },
                )
            }),
            DateTime::from_timestamp(self.9, 0).unwrap(),
            serde_json::from_str(&self.10)?,
            Vec::new(),
        ))
    }
}

type RecouringSignature = (i32, i64, Option<i64>);

impl From<Recouring> for RecouringSignature {
    fn from(val: Recouring) -> Self {
        match val {
            Recouring::DayInMonth(num) => (1, num as i64, None),
            Recouring::Days(datetime, days) => (2, datetime.timestamp(), Some(days as i64)),
            Recouring::Yearly(num1, num2) => (3, num1 as i64, Some(num2 as i64)),
        }
    }
}

impl TryFrom<RecouringSignature> for Recouring {
    type Error = anyhow::Error;

    fn try_from(value: RecouringSignature) -> Result<Self> {
        match value.0 {
            1 => Ok(Recouring::DayInMonth(value.1 as u16)),
            2 => Ok(Recouring::Days(
                DateTime::from_timestamp(value.1, 0).unwrap(),
                value.2.unwrap() as usize,
            )),
            3 => Ok(Recouring::Yearly(value.1 as u8, value.2.unwrap() as u16)),
            _ => anyhow::bail!("invalid id"),
        }
    }
}

type BudgetSignature = (Id, String, Option<String>, f64, i32, i32, i64, Option<i64>);

impl TryInto<Budget> for BudgetSignature {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Budget> {
        Ok(Budget::new(
            self.0,
            self.1,
            self.2,
            Currency::from_currency_id(self.4, self.3)?,
            Recouring::try_from((self.5, self.6, self.7))?,
        ))
    }
}

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
        connection.execute_batch(include_str!("schema.sql"))?;
        Ok(())
    }

    fn connect(&self) -> Result<rusqlite::Connection> {
        rusqlite::Connection::open(&self.path).context("failed to open database")
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl super::PrivateFinanceManager for SqliteFinanceManager {
    async fn private_create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT INTO asset_account (name, notes, iban, bic, offset_value, offset_currency) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            (&name, &note, &iban, &bic, offset.get_num(), offset.get_currency_id()),
        )?;
        connection.execute(
            "INSERT INTO account (asset_account) VALUES (?1)",
            (connection.last_insert_rowid(),),
        )?;
        Ok(AssetAccount::new(
            connection.last_insert_rowid() as u64,
            name,
            note,
            iban,
            bic,
            offset,
        ))
    }

    async fn private_update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        let connection = self.connect()?;

        let asset_account_id = get_asset_account_id(&connection, id)?;

        connection.execute(
            "UPDATE asset_account SET name=?1, notes=?2, iban=?3, bic=?4, offset_value=?5, offset_currency=?6 WHERE id=?7",
            (&name, &note, &iban, &bic, offset.get_num(), offset.get_currency_id(), asset_account_id),
        )?;
        Ok(account::AssetAccount::new(
            id, name, note, iban, bic, offset,
        ))
    }

    async fn private_create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::BookCheckingAccount> {
        let connection = self.connect()?;
        create_book_checking_account(&connection, name, notes, iban, bic)
    }

    async fn private_update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::BookCheckingAccount> {
        let connection = self.connect()?;
        let account_id = get_book_checking_account_id(&connection, id)?;
        connection.execute(
            "UPDATE book_checking_account SET name=?1, notes=?2, iban=?3, bic=?4 WHERE id=?5",
            (&name, &notes, &iban, &bic, account_id),
        )?;
        Ok(account::BookCheckingAccount::new(
            id, name, notes, iban, bic,
        ))
    }

    async fn private_get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> Result<Currency> {
        let connection = self.connect()?;

        let mut sum = Currency::Eur(0.0);

        // get negative number
        let negative_result: Vec<std::result::Result<(f64, i32), rusqlite::Error>> = connection
            .prepare(
                "SELECT SUM(amount_value), currency FROM transactions WHERE source_id=?1 AND timestamp < ?2 GROUP BY currency",
            )?
            .query_map((account.id(), date.timestamp()), |row| (Ok((row.get(0)?, row.get(1)?))))?
            .collect();
        for result in negative_result {
            let result = result?;
            sum -= Currency::from_currency_id(result.1, result.0)?;
        }
        // get positive number
        let positive_result: Vec<std::result::Result<(f64, i32), rusqlite::Error>> = connection
            .prepare(
                "SELECT SUM(amount_value), currency FROM transactions WHERE destination_id=?1 AND timestamp < ?2 GROUP BY currency",
            )?
            .query_map((account.id(), date.timestamp()), |row| (Ok((row.get(0)?, row.get(1)?))))?
            .collect();
        for result in positive_result {
            let result = result?;
            sum += Currency::from_currency_id(result.1, result.0)?;
        }

        Ok(sum)
    }
}

impl FinanceManager for SqliteFinanceManager {
    async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        let connection = self.connect()?;

        let mut accounts = Vec::new();

        let mut account_statement =
            connection.prepare("SELECT id, asset_account, book_checking_account FROM account")?;

        let account_rows: Vec<std::result::Result<Id, rusqlite::Error>> =
            account_statement.query_map((), |x| x.get(0))?.collect();

        for account_row in account_rows {
            let id = account_row?;
            accounts.push(get_account(&connection, id)?);
        }

        Ok(accounts)
    }

    async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        let connection = self.connect()?;
        Ok(Some(get_account(&connection, id)?))
    }

    async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        let connection = self.connect()?;
        let result: TransactionSignature = connection.query_row(
            format!(
                "SELECT {} FROM transactions WHERE id=?1",
                TRANSACTION_FIELDS
            )
            .as_str(),
            (&id,),
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                ))
            },
        )?;
        let categories = get_categories_of_transaction(&connection, result.0)?
            .iter()
            .map(|x| (*x.0.id(), x.1))
            .collect();
        let transaction: Transaction = result.try_into()?;
        Ok(Some(Transaction::new(
            *transaction.id(),
            transaction.amount(),
            transaction.title().to_owned(),
            transaction.description().map(|x| x.to_string()),
            *transaction.source(),
            *transaction.destination(),
            transaction.budget().copied(),
            *transaction.date(),
            transaction.metadata().clone(),
            categories,
        )))
    }

    async fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect()?;

        macro_rules! transaction_query {
            ($sql:expr, $params:expr) => {
                connection
                    .prepare($sql)?
                    .query_map($params, |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                            row.get(9)?,
                            row.get(10)?,
                        ))
                    })?
                    .collect()
            };
        }

        let result: Vec<std::result::Result<TransactionSignature, rusqlite::Error>> = match timespan {
            (None, None) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE source_id=?1 OR destination_id=?2", TRANSACTION_FIELDS).as_str(),
                (account, account)
            ),
            (Some(start), None) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE (source_id=?1 OR destination_id=?2) AND timestamp >= ?3", TRANSACTION_FIELDS).as_str(),
                (account, account, start.timestamp())
            ),
            (None, Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE (source_id=?1 OR destination_id=?2) AND timestamp <= ?3", TRANSACTION_FIELDS).as_str(),
                (account, account, end.timestamp())
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE (source_id=?1 OR destination_id=?2) AND timestamp >= ?3 AND timestamp <= ?4", TRANSACTION_FIELDS).as_str(),
                (account, account, start.timestamp(), end.timestamp())
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let transaction: Transaction = row?.try_into()?;
            let categories = get_categories_of_transaction(&connection, *transaction.id())?
                .iter()
                .map(|x| (*x.0.id(), x.1))
                .collect();
            transactions.push(Transaction::new(
                *transaction.id(),
                transaction.amount(),
                transaction.title().to_owned(),
                transaction.description().map(|x| x.to_string()),
                *transaction.source(),
                *transaction.destination(),
                transaction.budget().copied(),
                *transaction.date(),
                transaction.metadata().clone(),
                categories,
            ));
        }

        Ok(transactions)
    }

    async fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<(Id, Sign)>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categories: Vec<(Id, Sign)>,
    ) -> Result<Transaction> {
        let connection = self.connect()?;

        let source = match source {
            Or::One(id) => id,
            Or::Two(name) => {
                create_book_checking_account(&connection, name, None, None, None)?.id()
            }
        };

        let destination = match destination {
            Or::One(id) => id,
            Or::Two(name) => {
                create_book_checking_account(&connection, name, None, None, None)?.id()
            }
        };

        connection.execute(
            "
            INSERT INTO transactions (
                amount_value,
                currency,
                title,
                description,
                source_id,
                destination_id,
                budget,
                budget_sign,
                timestamp,
                metadata
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
            )
            ",
            (
                amount.get_num(),
                amount.get_currency_id(),
                &title,
                &description,
                &source,
                &destination,
                &budget.map(|x| x.0),
                &budget.map(|x| match x.1 {
                    Sign::Positive => true,
                    Sign::Negative => false,
                }),
                &date.timestamp(),
                serde_json::to_string(&metadata)?,
            ),
        )?;
        let transaction_id = connection.last_insert_rowid();

        set_categories_for_transaction(&connection, transaction_id as Id, &categories)?; // set categories for transaction

        Ok(Transaction::new(
            transaction_id as Id,
            amount,
            title,
            description,
            source,
            destination,
            budget,
            date,
            metadata,
            categories,
        ))
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> Result<Budget> {
        let connection = self.connect()?;

        let timespan_tuple = Into::<(i32, i64, Option<i64>)>::into(timespan.clone());

        connection.execute(
            "INSERT INTO budget (
                name,
                description,
                value,
                currency,
                timespan_type,
                timespan_field1,
                timespan_field2
            ) VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7
            )",
            (
                &name,
                &description,
                total_value.get_num(),
                total_value.get_currency_id(),
                timespan_tuple.0,
                timespan_tuple.1,
                timespan_tuple.2,
            ),
        )?;
        Ok(Budget::new(
            connection.last_insert_rowid() as Id,
            name,
            description,
            total_value,
            timespan,
        ))
    }

    async fn get_budgets(&self) -> Result<Vec<Budget>> {
        let connection = self.connect()?;

        let results: Vec<std::result::Result<BudgetSignature, rusqlite::Error>> = connection.prepare(
            "SELECT id, name, description, value, currency, timespan_type, timespan_field1, timespan_field2 FROM budget"
            )?.
            query_map((), |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)))?
            .collect();

        let mut budgets = Vec::new();

        for row in results {
            let row = row?;
            budgets.push(row.try_into()?);
        }
        Ok(budgets)
    }

    async fn get_budget(&self, id: Id) -> Result<Option<Budget>> {
        let connection = self.connect()?;

        let result: BudgetSignature = connection.query_row(
            "SELECT id, name, description, value, currency, timespan_type, timespan_field1, timespan_field2 FROM budget WHERE id=?1", 
            (&id,),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
        )?;

        Ok(Some(result.try_into()?))
    }

    async fn update_transaction(
        &mut self,
        id: Id,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>,
        destination: Or<Id, String>,
        budget: Option<(Id, Sign)>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categories: Vec<(Id, Sign)>,
    ) -> Result<Transaction> {
        let connection = self.connect()?;

        let source_id = match source {
            Or::One(id) => id,
            Or::Two(name) => {
                let account = self
                    .create_book_checking_account(name, None, None, None)
                    .await?;
                account.id()
            }
        };

        let destination_id = match destination {
            Or::One(id) => id,
            Or::Two(name) => {
                let account = self
                    .create_book_checking_account(name, None, None, None)
                    .await?;
                account.id()
            }
        };

        connection.execute(
            "UPDATE transactions SET amount_value=?1, currency=?2, title=?3, description=?4, source_id=?5, destination_id=?6, budget=?7, budget_sign=?8, timestamp=?9, metadata=?10 WHERE id=?11", 
            (amount.get_num(), amount.get_currency_id(), &title, &description, source_id, destination_id, budget.map(|x| x.0), budget.map(|x| match x.1 {Sign::Positive => true, Sign::Negative => false}), date.timestamp(), serde_json::to_string(&metadata)?, id)
        )?;

        set_categories_for_transaction(&connection, id, &categories)?; // set categories for transaction

        Ok(Transaction::new(
            id,
            amount,
            title,
            description,
            source_id,
            destination_id,
            budget,
            date,
            metadata,
            categories,
        ))
    }

    async fn delete_transaction(&mut self, id: Id) -> Result<()> {
        let connection = self.connect()?;
        connection.execute("DELETE FROM transactions WHERE id=?1", (id,))?;
        connection.execute(
            "DELETE FROM transaction_category WHERE transaction_id=?1",
            (id,),
        )?;
        Ok(())
    }

    async fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect()?;

        macro_rules! transaction_query {
            ($sql:expr, $params:expr) => {
                connection
                    .prepare($sql)?
                    .query_map($params, |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                            row.get(9)?,
                            row.get(10)?,
                        ))
                    })?
                    .collect()
            };
        }

        let result: Vec<std::result::Result<TransactionSignature, rusqlite::Error>> = match timespan {
            (None, None) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1", TRANSACTION_FIELDS).as_str(),
                (id,)
            ),
            (Some(start), None) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1 AND timestamp >= ?2", TRANSACTION_FIELDS).as_str(),
                (id, start.timestamp())
            ),
            (None, Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1 AND timestamp <= ?2", TRANSACTION_FIELDS).as_str(),
                (id, end.timestamp())
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1 AND timestamp >= ?2 AND timestamp <= ?3", TRANSACTION_FIELDS).as_str(),
                (id, start.timestamp(), end.timestamp())
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let transaction: Transaction = row?.try_into()?;
            let categories = get_categories_of_transaction(&connection, *transaction.id())?
                .iter()
                .map(|x| (*x.0.id(), x.1))
                .collect();
            transactions.push(Transaction::new(
                *transaction.id(),
                transaction.amount(),
                transaction.title().to_owned(),
                transaction.description().map(|x| x.to_string()),
                *transaction.source(),
                *transaction.destination(),
                transaction.budget().copied(),
                *transaction.date(),
                transaction.metadata().clone(),
                categories,
            ));
        }

        Ok(transactions)
    }

    async fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> Result<Budget> {
        let connection = self.connect()?;
        let timespan_tuple = Into::<(i32, i64, Option<i64>)>::into(timespan.clone());

        connection.execute(
                "UPDATE budget SET name=?1, description=?2, value=?3, currency=?4, timespan_type=?5, timespan_field1=?6, timespan_field2=?7 WHERE id=?8",
                (
                    &name,
                    &description,
                    total_value.get_num(),
                    total_value.get_currency_id(),
                    timespan_tuple.0,
                    timespan_tuple.1,
                    timespan_tuple.2,
                    id,
                ),
            )?;
        Ok(Budget::new(id, name, description, total_value, timespan))
    }

    async fn get_transactions(&self, timespan: Timespan) -> Result<Vec<Transaction>> {
        let connection = self.connect()?;

        macro_rules! transaction_query {
            ($sql:expr, $params:expr) => {
                connection
                    .prepare($sql)?
                    .query_map($params, |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                            row.get(9)?,
                            row.get(10)?,
                        ))
                    })?
                    .collect()
            };
        }

        let result: Vec<std::result::Result<TransactionSignature, rusqlite::Error>> = match timespan
        {
            (None, None) => transaction_query!(
                format!("SELECT {} FROM transactions", TRANSACTION_FIELDS).as_str(),
                ()
            ),
            (Some(start), None) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions WHERE timestamp >= ?1",
                    TRANSACTION_FIELDS
                )
                .as_str(),
                (start.timestamp(),)
            ),
            (None, Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions WHERE timestamp <= ?1",
                    TRANSACTION_FIELDS
                )
                .as_str(),
                (end.timestamp(),)
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions WHERE timestamp >= ?1 AND timestamp <= ?2",
                    TRANSACTION_FIELDS
                )
                .as_str(),
                (start.timestamp(), end.timestamp())
            ),
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let transaction: Transaction = row?.try_into()?;
            let categories = get_categories_of_transaction(&connection, *transaction.id())?
                .iter()
                .map(|x| (*x.0.id(), x.1))
                .collect();
            transactions.push(Transaction::new(
                *transaction.id(),
                transaction.amount(),
                transaction.title().to_owned(),
                transaction.description().map(|x| x.to_string()),
                *transaction.source(),
                *transaction.destination(),
                transaction.budget().copied(),
                *transaction.date(),
                transaction.metadata().clone(),
                categories,
            ));
        }

        Ok(transactions)
    }

    async fn get_categories(&self) -> Result<Vec<Category>> {
        let connection = self.connect()?;
        let mut categories = Vec::new();
        let mut statement = connection.prepare("SELECT id, name FROM categories")?;
        let rows: Vec<std::result::Result<(Id, String), rusqlite::Error>> = statement
            .query_map((), |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect();
        for row in rows {
            let row = row?;
            categories.push(Category::new(row.0, row.1));
        }
        Ok(categories)
    }

    async fn create_category(&mut self, name: String) -> Result<Category> {
        let connection = self.connect()?;
        connection.execute("INSERT INTO categories (name) VALUES (?1)", (&name,))?;
        Ok(Category::new(connection.last_insert_rowid() as Id, name))
    }

    async fn update_category(&mut self, id: Id, name: String) -> Result<Category> {
        let connection = self.connect()?;
        connection.execute("UPDATE categories SET name=?1 WHERE id=?2", (&name, id))?;
        Ok(Category::new(id, name))
    }

    async fn get_category(&self, id: Id) -> Result<Option<Category>> {
        let connection = self.connect()?;
        get_category(&connection, id)
    }

    async fn delete_category(&mut self, id: Id) -> Result<()> {
        let connection = self.connect()?;
        connection.execute("DELETE FROM categories WHERE id=?1", (id,))?;
        connection.execute(
            "DELETE FROM transaction_category WHERE category_id=?1",
            (id,),
        )?; // delete all references to the category
        Ok(())
    }

    async fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect()?;

        macro_rules! transaction_query {
            ($sql:expr, $params:expr) => {
                connection
                    .prepare($sql)?
                    .query_map($params, |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                            row.get(8)?,
                            row.get(9)?,
                            row.get(10)?,
                        ))
                    })?
                    .collect()
            };
        }

        let result: Vec<std::result::Result<TransactionSignature, rusqlite::Error>> = match timespan {
            (None, None) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE category_id=?1",
                    TRANSACTION_FIELDS
                ).as_str(),
                (&id,)
            ),
            (Some(start), None) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE timestamp >= ?1 AND category_id=?2",
                    TRANSACTION_FIELDS
                ).as_str(),
                (start.timestamp(), &id)
            ),
            (None, Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE timestamp <= ?1  AND category_id=?2",
                    TRANSACTION_FIELDS
                ).as_str(),
                (end.timestamp(), &id)
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE timestamp >= ?1 AND timestamp <= ?2  AND category_id=?3",
                    TRANSACTION_FIELDS
                ).as_str(),
                (start.timestamp(), end.timestamp(), &id)
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let transaction: Transaction = row?.try_into()?;
            let categories = get_categories_of_transaction(&connection, *transaction.id())?
                .iter()
                .map(|x| (*x.0.id(), x.1))
                .collect();
            transactions.push(Transaction::new(
                *transaction.id(),
                transaction.amount(),
                transaction.title().to_owned(),
                transaction.description().map(|x| x.to_string()),
                *transaction.source(),
                *transaction.destination(),
                transaction.budget().copied(),
                *transaction.date(),
                transaction.metadata().clone(),
                categories,
            ));
        }

        Ok(transactions)
    }
}

fn get_asset_account_id(connection: &rusqlite::Connection, account_id: Id) -> Result<i32> {
    let result = connection.query_row(
        "SELECT asset_account FROM account WHERE id=?1",
        (account_id,),
        |row| row.get(0),
    )?;
    match result {
        Some(id) => Ok(id),
        None => anyhow::bail!("can not find asset account"),
    }
}

fn get_book_checking_account_id(connection: &rusqlite::Connection, account_id: Id) -> Result<i32> {
    let result = connection.query_row(
        "SELECT book_checking_account FROM account WHERE id=?1",
        (account_id,),
        |row| row.get(0),
    )?;
    match result {
        Some(id) => Ok(id),
        None => anyhow::bail!("can not find asset account"),
    }
}

fn get_account(connection: &rusqlite::Connection, account_id: Id) -> Result<account::Account> {
    let account_result: (Option<Id>, Option<Id>) = connection.query_row(
        "SELECT asset_account, book_checking_account FROM account WHERE id=?1",
        (account_id,),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if let Some(id) = account_result.0 {
        let asset_account_result: (String, Option<String>, Option<String>, Option<String>, f64, i32) =
            connection.query_row(
                "SELECT name, notes, iban, bic, offset_value, offset_currency FROM asset_account WHERE id=?1",
                (id,),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )?;
        Ok(account::AssetAccount::new(
            account_id,
            asset_account_result.0,
            asset_account_result.1,
            asset_account_result.2,
            asset_account_result.3,
            Currency::from_currency_id(asset_account_result.5, asset_account_result.4)?,
        )
        .into())
    } else if let Some(id) = account_result.1 {
        let book_checking_account_result: (String, Option<String>, Option<String>, Option<String>) =
            connection.query_row(
                "SELECT name, notes, iban, bic FROM book_checking_account WHERE id=?1",
                (id,),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;
        Ok(account::BookCheckingAccount::new(
            account_id,
            book_checking_account_result.0,
            book_checking_account_result.1,
            book_checking_account_result.2,
            book_checking_account_result.3,
        )
        .into())
    } else {
        anyhow::bail!("could not find the account");
    }
}

fn create_book_checking_account(
    connection: &rusqlite::Connection,
    name: String,
    notes: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
) -> Result<account::BookCheckingAccount> {
    connection.execute(
        "INSERT INTO book_checking_account (name, notes, iban, bic) VALUES (?1, ?2, ?3, ?4)",
        (&name, &notes, &iban, &bic),
    )?;
    connection.execute(
        "INSERT INTO account (book_checking_account) VALUES (?1)",
        (connection.last_insert_rowid(),),
    )?;
    Ok(account::BookCheckingAccount::new(
        connection.last_insert_rowid() as Id,
        name,
        notes,
        iban,
        bic,
    ))
}

fn set_categories_for_transaction(
    connection: &rusqlite::Connection,
    transaction_id: Id,
    categories: &Vec<(Id, Sign)>,
) -> Result<()> {
    connection.execute(
        "DELETE FROM transaction_category WHERE transaction_id=?1",
        (transaction_id,),
    )?;
    for category in categories {
        connection.execute(
            "INSERT INTO transaction_category (transaction_id, category_id, sign) VALUES (?1, ?2, ?3)",
            (transaction_id, category.0, category.1 == Sign::Positive),
        )?;
    }
    Ok(())
}

fn get_category(connection: &rusqlite::Connection, category_id: Id) -> Result<Option<Category>> {
    let result: Option<String> = connection.query_row(
        "SELECT name FROM categories WHERE id=?1",
        (&category_id,),
        |row| row.get(0),
    )?;
    match result {
        Some(name) => Ok(Some(Category::new(category_id, name))),
        None => Ok(None),
    }
}

fn get_categories_of_transaction(
    connection: &rusqlite::Connection,
    transaction_id: Id,
) -> Result<Vec<(Category, Sign)>> {
    let mut categories = Vec::new();
    let mut statement = connection
        .prepare("SELECT category_id, sign FROM transaction_category WHERE transaction_id=?1")?;
    let rows: Vec<std::result::Result<(Id, bool), rusqlite::Error>> = statement
        .query_map((transaction_id,), |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect();
    for row in rows {
        let row = row?;
        categories.push((
            get_category(connection, row.0)?.unwrap(),
            if row.1 {
                Sign::Positive
            } else {
                Sign::Negative
            },
        ));
    }
    Ok(categories)
}
