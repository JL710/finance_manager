use crate::account::AssetAccount;

use crate::*;
use anyhow::{Context, Result};
use bigdecimal::{BigDecimal, FromPrimitive};
use rusqlite::{OptionalExtension, fallible_iterator::FallibleIterator};

use async_std::sync::{Mutex, MutexGuard};
use std::sync::Arc;

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
        Transaction::new(
            self.0,
            Currency::from_currency_id(self.2, BigDecimal::from_f64(self.1).unwrap())?,
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
            DateTime::from_unix_timestamp(self.9).unwrap(),
            serde_json::from_str(&self.10)?,
            HashMap::new(),
        )
    }
}

type RecurringSignature = (i32, i64, Option<i64>);

impl From<budget::Recurring> for RecurringSignature {
    fn from(val: budget::Recurring) -> Self {
        match val {
            budget::Recurring::DayInMonth(num) => (1, num as i64, None),
            budget::Recurring::Days(datetime, days) => {
                (2, datetime.unix_timestamp(), Some(days as i64))
            }
            budget::Recurring::Yearly(num1, num2) => (3, num1 as i64, Some(num2 as i64)),
        }
    }
}

impl TryFrom<RecurringSignature> for budget::Recurring {
    type Error = anyhow::Error;

    fn try_from(value: RecurringSignature) -> Result<Self> {
        match value.0 {
            1 => Ok(budget::Recurring::DayInMonth(value.1 as u8)),
            2 => Ok(budget::Recurring::Days(
                DateTime::from_unix_timestamp(value.1).unwrap(),
                value.2.unwrap() as usize,
            )),
            3 => Ok(budget::Recurring::Yearly(
                value.1 as u8,
                value.2.unwrap() as u8,
            )),
            _ => anyhow::bail!("invalid id"),
        }
    }
}

type BudgetSignature = (Id, String, Option<String>, f64, i32, i32, i64, Option<i64>);

impl TryFrom<BudgetSignature> for Budget {
    type Error = anyhow::Error;

    fn try_from(value: BudgetSignature) -> std::result::Result<Self, Self::Error> {
        Ok(Budget::new(
            value.0,
            value.1,
            value.2,
            Currency::from_currency_id(value.4, BigDecimal::from_f64(value.3).unwrap())?,
            budget::Recurring::try_from((value.5, value.6, value.7))?,
        ))
    }
}

type BillSignature = (Id, String, Option<String>, f64, i32, Option<i64>, bool);

impl TryInto<Bill> for BillSignature {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Bill> {
        Ok(Bill::new(
            self.0,
            self.1,
            self.2,
            Currency::from_currency_id(self.4, BigDecimal::from_f64(self.3).unwrap())?,
            HashMap::new(),
            if let Some(timestamp) = self.5 {Some(time::OffsetDateTime::from_unix_timestamp(timestamp).unwrap())} else {None},
            self.6
        ))
    }
}

async fn migrate_db(connection: MutexGuard<'_, rusqlite::Connection>) -> Result<()> {
    let version_result: Option<i32> = connection
        .query_row(
            "SELECT value FROM database_info WHERE tag='version'",
            (),
            |x| x.get(0),
        )
        .optional()
        .unwrap()
        .map(|x: String| x.parse().unwrap());

    if let Some(version) = version_result {
        match version {
            0 => {
                connection.execute("ALTER TABLE bill ADD closed BOOLEAN NOT NULL DEFAULT false;", ())?;
                connection.execute("UPDATE database_info SET value=1 WHERE tag='version'", ())?;
            }
            1 => {}
            _ => panic!("unknown database version"),
        }
    } else {
        connection.execute(
            "INSERT INTO database_info (tag, value) VALUES ('version', '1')",
            (),
        )?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct SqliteFinanceManager {
    path: String,
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteFinanceManager {
    async fn init_db(&self) -> Result<()> {
        let connection = self.connect().await;
        connection.execute_batch(include_str!("schema.sql"))?;
        migrate_db(connection).await?;
        Ok(())
    }

    async fn connect(&self) -> async_std::sync::MutexGuard<rusqlite::Connection> {
        self.connection.lock().await
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn new_in_memory() -> Result<Self> {
        let new = Self {
            connection: Arc::new(Mutex::new(rusqlite::Connection::open_in_memory()?)),
            path: String::new(),
        };
        async_std::task::block_on(async { new.init_db().await })?;
        Ok(new)
    }
}

impl FinanceManager for SqliteFinanceManager {
    type Flags = String;

    fn new(path: Self::Flags) -> Result<Self> {
        let new = Self {
            connection: Arc::new(Mutex::new(rusqlite::Connection::open(&path)?)),
            path,
        };
        async_std::task::block_on(async { new.init_db().await })?;
        Ok(new)
    }

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        let connection = self.connect().await;
        connection.execute(
            "INSERT INTO asset_account (name, notes, iban, bic, offset_value, offset_currency) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            (&name, &note, iban.clone().map(|x| x.electronic_str().to_owned()), bic.as_ref().map(|x|x.to_string()), offset.get_eur_num(), offset.get_currency_id()),
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

    async fn update_asset_account(
        &mut self,
        account: account::AssetAccount,
    ) -> Result<account::AssetAccount> {
        let connection = self.connect().await;

        let asset_account_id = get_asset_account_id(&connection, account.id)?;

        connection.execute(
            "UPDATE asset_account SET name=?1, notes=?2, iban=?3, bic=?4, offset_value=?5, offset_currency=?6 WHERE id=?7",
            (
                &account.name, 
                &account.note, 
                account.iban.clone().map(|x| x.electronic_str().to_owned()), 
                account.bic.as_ref().map(|x|x.to_string()), 
                account.offset.get_eur_num(), 
                account.offset.get_currency_id(), 
                asset_account_id
            ),
        )?;
        Ok(account)
    }

    async fn delete_account(&mut self, id: Id) -> Result<()> {
        let connection = self.connect().await;

        let account_result: (Option<Id>, Option<Id>) = connection
            .query_row(
                "SELECT asset_account, book_checking_account FROM account WHERE id=?1",
                (id,),
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .context("could not query entry from account table")?;
        connection.execute("DELETE FROM account WHERE id=?1", (id,))?;
        match account_result {
            (Some(asset_account_id), None) => {
                connection
                    .execute("DELETE FROM asset_account WHERE id=?1", (asset_account_id,))
                    .context("could not delete from asset_account table")?;
            }
            (None, Some(book_checking_account_id)) => {
                connection
                    .execute(
                        "DELETE FROM book_checking_account WHERE id=?1",
                        (book_checking_account_id,),
                    )
                    .context("could not delete from book_checking_account table")?;
            }
            _ => anyhow::bail!("can not find account with id {}", id),
        }
        Ok(())
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
    ) -> Result<account::BookCheckingAccount> {
        let connection = self.connect().await;
        create_book_checking_account(&connection, name, notes, iban, bic)
    }

    async fn update_book_checking_account(
        &mut self,
        account: account::BookCheckingAccount
    ) -> Result<account::BookCheckingAccount> {
        let connection = self.connect().await;
        let account_id = get_book_checking_account_id(&connection, account.id)?;
        connection.execute(
            "UPDATE book_checking_account SET name=?1, notes=?2, iban=?3, bic=?4 WHERE id=?5",
            (
                &account.name,
                &account.note,
                account.iban.clone().map(|x| x.electronic_str().to_owned()),
                account.bic.as_ref().map(|x| x.to_string()),
                account_id,
            ),
        )?;
        Ok(account)
    }

    async fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
        closed: bool
    ) -> Result<Bill> {
        let connection = self.connect().await;

        connection.execute(
            "INSERT INTO bill (name, description, value, value_currency, due_date, closed) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &name,
                &description,
                value.get_eur_num(),
                value.get_currency_id(),
                due_date.map(|x| x.unix_timestamp()),
                closed
            ),
        )?;
        let bill_id = connection.last_insert_rowid();

        for transaction_pair in &transactions {
            connection.execute(
                "INSERT INTO bill_transaction (bill_id, transaction_id, sign) VALUES (?1, ?2, ?3)",
                (
                    bill_id,
                    transaction_pair.0,
                    *transaction_pair.1 == Sign::Positive,
                ),
            )?;
        }

        Ok(Bill::new(
            bill_id as Id,
            name,
            description,
            value,
            transactions,
            due_date,
            closed
        ))
    }

    async fn update_bill(&mut self, bill: Bill) -> Result<()> {
        let connection = self.connect().await;

        connection.execute(
            "UPDATE bill SET name=?1, description=?2, value=?3, value_currency=?4, due_date=?5, closed=?6 WHERE id=?7",
            (
                &bill.name,
                bill.description,
                bill.value.get_eur_num(),
                bill.value.get_currency_id(),
                bill.due_date.map(|x| x.unix_timestamp()),
                bill.closed,
                bill.id,
            ),
        )?;

        connection.execute("DELETE FROM bill_transaction WHERE bill_id=?1", (bill.id,))?;

        for transaction_pair in &bill.transactions {
            connection.execute(
                "INSERT INTO bill_transaction (bill_id, transaction_id, sign) VALUES (?1, ?2, ?3)",
                (
                    bill.id,
                    transaction_pair.0,
                    *transaction_pair.1 == Sign::Positive,
                ),
            )?;
        }

        Ok(())
    }

    async fn get_bills(&self, closed: Option<bool>) -> Result<Vec<Bill>> {
        let connection = self.connect().await;

        let mut bills = Vec::new();

        let mut sql = "SELECT id, name, description, value, value_currency, due_date, closed FROM bill".to_string();
        if closed.is_some() {
            sql += " WHERE closed=?1";
        }

        let mut stmt = connection.prepare(&sql)?;
        
        let query = if let Some(closed) = closed {
            stmt.query((closed,))?
        } else {
            stmt.query(())?
        };

        let bills_result: Vec<BillSignature> = query.map(|x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?, x.get(4)?, x.get(5)?, x.get(6)?))).collect()?;

        for bill_result in bills_result {
            let mut bill: Bill = bill_result.try_into()?;

            bill.transactions = get_transactions_of_bill(&connection, bill.id)?;
            bills.push(bill);
        }

        Ok(bills)
    }

    async fn get_bill(&self, id: &Id) -> Result<Option<Bill>> {
        let connection = self.connect().await;

        let bill_result: BillSignature = connection.query_row(
            "SELECT id, name, description, value, value_currency, due_date, closed FROM bill WHERE id=? ",
            (id,),
            |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?, x.get(3)?, x.get(4)?, x.get(5)?, x.get(6)?)),
        )?;

        let mut bill: Bill = bill_result.try_into()?;

        bill.transactions = get_transactions_of_bill(&connection, bill.id)?;

        Ok(Some(bill))
    }

    async fn delete_bill(&mut self, id: Id) -> Result<()> {
        let connection = self.connect().await;

        connection.execute("DELETE FROM bill_transaction WHERE bill_id=?1", (id,))?;
        connection.execute("DELETE FROM bill WHERE id=?1", (id,))?;
        Ok(())
    }

    async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        let connection = self.connect().await;

        let mut accounts = Vec::new();

        let mut account_statement =
            connection.prepare("SELECT id, asset_account, book_checking_account FROM account")?;

        let account_rows: Vec<std::result::Result<Id, rusqlite::Error>> =
            account_statement.query_map((), |x| x.get(0))?.collect();

        for account_row in account_rows {
            let id = account_row?;
            accounts.push(get_account(&connection, id)?.unwrap());
        }

        Ok(accounts)
    }

    async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        let connection = self.connect().await;
        get_account(&connection, id)
    }

    async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        let connection = self.connect().await;
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
        let mut transaction: Transaction = result.try_into()?;
        transaction.categories = get_categories_of_transaction(&connection, transaction.id)?
            .iter()
            .map(|x| (x.0.id, x.1))
            .collect();
        Ok(Some(transaction))
    }

    async fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect().await;

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
                (account, account, start.unix_timestamp())
            ),
            (None, Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE (source_id=?1 OR destination_id=?2) AND timestamp <= ?3", TRANSACTION_FIELDS).as_str(),
                (account, account, end.unix_timestamp())
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE (source_id=?1 OR destination_id=?2) AND timestamp >= ?3 AND timestamp <= ?4", TRANSACTION_FIELDS).as_str(),
                (account, account, start.unix_timestamp(), end.unix_timestamp())
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let mut transaction: Transaction = row?.try_into()?;
            transaction.categories = get_categories_of_transaction(&connection, transaction.id)?
                .iter()
                .map(|x| (x.0.id, x.1))
                .collect();
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    async fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Id,
        destination: Id,
        budget: Option<(Id, Sign)>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categories: HashMap<Id, Sign>,
    ) -> Result<Transaction> {
        let connection = self.connect().await;

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
                amount.get_eur_num(),
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
                &date.unix_timestamp(),
                serde_json::to_string(&metadata)?,
            ),
        )?;
        let transaction_id = connection.last_insert_rowid();

        set_categories_for_transaction(&connection, transaction_id as Id, &categories)?; // set categories for transaction

        Transaction::new(
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
        )
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: budget::Recurring,
    ) -> Result<Budget> {
        let connection = self.connect().await;

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
                total_value.get_eur_num(),
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

    async fn delete_budget(&mut self, id: Id) -> Result<()> {
        let connection = self.connect().await;

        connection.execute(
            "UPDATE transactions SET budget=null, budget_sign=null WHERE id=?1",
            (id,),
        )?;
        connection.execute("DELETE FROM budget WHERE id = ?1", (id,))?;

        Ok(())
    }

    async fn get_budgets(&self) -> Result<Vec<Budget>> {
        let connection = self.connect().await;

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
        let connection = self.connect().await;

        let result: Option<BudgetSignature> = connection.query_row(
            "SELECT id, name, description, value, currency, timespan_type, timespan_field1, timespan_field2 FROM budget WHERE id=?1", 
            (&id,),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
        ).optional()?;

        if let Some(x) = result {
            Ok(Some(x.try_into()?))
        } else {
            Ok(None)
        }
    }

    async fn update_transaction(&mut self, transaction: Transaction) -> Result<Transaction> {
        let connection = self.connect().await;

        connection.execute(
            "UPDATE transactions SET amount_value=?1, currency=?2, title=?3, description=?4, source_id=?5, destination_id=?6, budget=?7, budget_sign=?8, timestamp=?9, metadata=?10 WHERE id=?11", 
            (
                transaction.amount().get_eur_num(),
                transaction.amount().get_currency_id(),
                &transaction.title,
                &transaction.description,
                transaction.source,
                transaction.destination,
                transaction.budget.map(|x| x.0),
                transaction.budget.map(|x| match x.1 {Sign::Positive => true, Sign::Negative => false}),
                transaction.date.unix_timestamp(),
                serde_json::to_string(&transaction.metadata)?,
                transaction.id
            )
        )?;

        set_categories_for_transaction(&connection, transaction.id, &transaction.categories)?; // set categories for transaction

        Ok(transaction)
    }

    async fn delete_transaction(&mut self, id: Id) -> Result<()> {
        let connection = self.connect().await;
        connection.execute(
            "DELETE FROM transaction_category WHERE transaction_id=?1",
            (id,),
        )?;
        connection.execute(
            "DELETE FROM bill_transaction WHERE transaction_id=?1",
            (id,),
        )?;
        connection.execute("DELETE FROM transactions WHERE id=?1", (id,))?;
        Ok(())
    }

    async fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect().await;

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
                (id, start.unix_timestamp())
            ),
            (None, Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1 AND timestamp <= ?2", TRANSACTION_FIELDS).as_str(),
                (id, end.unix_timestamp())
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!("SELECT {} FROM transactions WHERE budget=?1 AND timestamp >= ?2 AND timestamp <= ?3", TRANSACTION_FIELDS).as_str(),
                (id, start.unix_timestamp(), end.unix_timestamp())
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let mut transaction: Transaction = row?.try_into()?;
            transaction.categories = get_categories_of_transaction(&connection, transaction.id)?
                .iter()
                .map(|x| (x.0.id, x.1))
                .collect();
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    async fn update_budget(&mut self, budget: Budget) -> Result<Budget> {
        let connection = self.connect().await;
        let timespan_tuple = Into::<(i32, i64, Option<i64>)>::into(budget.timespan.clone());

        connection.execute(
                "UPDATE budget SET name=?1, description=?2, value=?3, currency=?4, timespan_type=?5, timespan_field1=?6, timespan_field2=?7 WHERE id=?8",
                (
                    &budget.name,
                    &budget.description,
                    budget.total_value.get_eur_num(),
                    budget.total_value.get_currency_id(),
                    timespan_tuple.0,
                    timespan_tuple.1,
                    timespan_tuple.2,
                    budget.id,
                ),
            )?;
        Ok(budget)
    }

    async fn get_transactions_in_timespan(&self, timespan: Timespan) -> Result<Vec<Transaction>> {
        let connection = self.connect().await;

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
                (start.unix_timestamp(),)
            ),
            (None, Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions WHERE timestamp <= ?1",
                    TRANSACTION_FIELDS
                )
                .as_str(),
                (end.unix_timestamp(),)
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions WHERE timestamp >= ?1 AND timestamp <= ?2",
                    TRANSACTION_FIELDS
                )
                .as_str(),
                (start.unix_timestamp(), end.unix_timestamp())
            ),
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let mut transaction: Transaction = row?.try_into()?;
            transaction.categories = get_categories_of_transaction(&connection, transaction.id)?
                .iter()
                .map(|x| (x.0.id, x.1))
                .collect();
            transactions.push(transaction);
        }

        Ok(transactions)
    }

    async fn get_categories(&self) -> Result<Vec<Category>> {
        let connection = self.connect().await;
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
        let connection = self.connect().await;
        connection.execute("INSERT INTO categories (name) VALUES (?1)", (&name,))?;
        Ok(Category::new(connection.last_insert_rowid() as Id, name))
    }

    async fn update_category(&mut self, category: Category) -> Result<Category> {
        let connection = self.connect().await;
        connection.execute(
            "UPDATE categories SET name=?1 WHERE id=?2",
            (&category.name, category.id),
        )?;
        Ok(category)
    }

    async fn get_category(&self, id: Id) -> Result<Option<Category>> {
        let connection = self.connect().await;
        get_category(&connection, id)
    }

    async fn delete_category(&mut self, id: Id) -> Result<()> {
        let connection = self.connect().await;
        connection.execute(
            "DELETE FROM transaction_category WHERE category_id=?1",
            (id,),
        )?; // delete all references to the category
        connection.execute("DELETE FROM categories WHERE id=?1", (id,))?;
        Ok(())
    }

    async fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let connection = self.connect().await;

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
                (start.unix_timestamp(), &id)
            ),
            (None, Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE timestamp <= ?1  AND category_id=?2",
                    TRANSACTION_FIELDS
                ).as_str(),
                (end.unix_timestamp(), &id)
            ),
            (Some(start), Some(end)) => transaction_query!(
                format!(
                    "SELECT {} FROM transactions INNER JOIN transaction_category ON transaction_id = id WHERE timestamp >= ?1 AND timestamp <= ?2  AND category_id=?3",
                    TRANSACTION_FIELDS
                ).as_str(),
                (start.unix_timestamp(), end.unix_timestamp(), &id)
            )
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in result {
            let mut transaction: Transaction = row?.try_into()?;
            transaction.categories = get_categories_of_transaction(&connection, transaction.id)?
                .iter()
                .map(|x| (x.0.id, x.1))
                .collect();
            transactions.push(transaction);
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

fn get_account(
    connection: &rusqlite::Connection,
    account_id: Id,
) -> Result<Option<account::Account>> {
    let account_result: (Option<Id>, Option<Id>) = match connection
        .query_row(
            "SELECT asset_account, book_checking_account FROM account WHERE id=?1",
            (account_id,),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .context("Error during id resolution")?
    {
        Some(x) => x,
        None => return Ok(None),
    };
    if let Some(id) = account_result.0 {
        let asset_account_result: (String, Option<String>, Option<String>, Option<String>, f64, i32) =
            connection.query_row(
                "SELECT name, notes, iban, bic, offset_value, offset_currency FROM asset_account WHERE id=?1",
                (id,),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )?;
        Ok(Some(
            account::AssetAccount::new(
                account_id,
                asset_account_result.0,
                asset_account_result.1,
                if let Some(iban_str) = asset_account_result.2 {
                    Some(iban_str.parse()?)
                } else {
                    None
                },
                asset_account_result.3.map(|x| x.into()),
                Currency::from_currency_id(
                    asset_account_result.5,
                    BigDecimal::from_f64(asset_account_result.4).unwrap(),
                )?,
            )
            .into(),
        ))
    } else if let Some(id) = account_result.1 {
        let book_checking_account_result: (String, Option<String>, Option<String>, Option<String>) =
            connection.query_row(
                "SELECT name, notes, iban, bic FROM book_checking_account WHERE id=?1",
                (id,),
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )?;
        Ok(Some(
            account::BookCheckingAccount::new(
                account_id,
                book_checking_account_result.0,
                book_checking_account_result.1,
                if let Some(iban_str) = book_checking_account_result.2 {
                    Some(iban_str.parse()?)
                } else {
                    None
                },
                book_checking_account_result.3.map(|x| x.into()),
            )
            .into(),
        ))
    } else {
        anyhow::bail!("could not find the account");
    }
}

fn create_book_checking_account(
    connection: &rusqlite::Connection,
    name: String,
    notes: Option<String>,
    iban: Option<AccountId>,
    bic: Option<Bic>,
) -> Result<account::BookCheckingAccount> {
    connection.execute(
        "INSERT INTO book_checking_account (name, notes, iban, bic) VALUES (?1, ?2, ?3, ?4)",
        (
            &name,
            &notes,
            iban.clone().map(|x| x.electronic_str().to_owned()),
            bic.as_ref().map(|x| x.to_string()),
        ),
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
    categories: &HashMap<Id, Sign>,
) -> Result<()> {
    connection.execute(
        "DELETE FROM transaction_category WHERE transaction_id=?1",
        (transaction_id,),
    )?;
    for category in categories {
        connection.execute(
            "INSERT INTO transaction_category (transaction_id, category_id, sign) VALUES (?1, ?2, ?3)",
            (transaction_id, category.0, *category.1 == Sign::Positive),
        )?;
    }
    Ok(())
}

fn get_category(connection: &rusqlite::Connection, category_id: Id) -> Result<Option<Category>> {
    let result: Option<String> = match connection.query_row(
        "SELECT name FROM categories WHERE id=?1",
        (&category_id,),
        |row| row.get(0),
    ) {
        Err(error) => match error {
            rusqlite::Error::QueryReturnedNoRows => None,
            _ => return Err(error.into()),
        },
        Ok(name) => Some(name),
    };
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

fn get_transactions_of_bill(
    connection: &rusqlite::Connection,
    bill_id: Id,
) -> Result<HashMap<Id, Sign>> {
    let mut statement =
        connection.prepare("SELECT transaction_id, sign FROM bill_transaction WHERE bill_id=?1")?;
    let rows: Vec<std::result::Result<(Id, bool), rusqlite::Error>> = statement
        .query_map((bill_id,), |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect();
    let mut transactions = HashMap::with_capacity(rows.len());
    for row in rows {
        let row = row?;
        transactions.insert(
            row.0,
            if row.1 {
                Sign::Positive
            } else {
                Sign::Negative
            },
        );
    }
    Ok(transactions)
}

#[cfg(test)]
mod test {
    async fn test_runner(test: impl AsyncFn(super::SqliteFinanceManager)) {
        test(super::SqliteFinanceManager::new_in_memory().unwrap()).await
    }

    crate::finance_manager_test::unit_tests!(test_runner);
}
