use crate::*;
use anyhow::{Context, Result};
use std::future::Future;

use async_std::sync::Mutex;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct FMController<FM>
where
    FM: FinanceManager + 'static,
{
    finance_manager: Arc<Mutex<FM>>,
}

impl<FM> FMController<FM>
where
    FM: FinanceManager,
{
    pub fn new(flags: FM::Flags) -> Result<Self> {
        Ok(Self {
            finance_manager: Arc::new(Mutex::new(FM::new(flags)?)),
        })
    }

    pub fn with_finance_manager(finance_manager: FM) -> Self {
        Self {
            finance_manager: Arc::new(Mutex::new(finance_manager)),
        }
    }

    pub fn raw_fm(&self) -> &Arc<Mutex<FM>> {
        &self.finance_manager
    }

    pub fn get_bill_sum<'a>(
        &'a self,
        bill: &'a Bill,
    ) -> impl Future<Output = Result<Currency>> + MaybeSend + 'a {
        let transactions_future =
            self.get_transactions(bill.transactions.keys().cloned().collect::<Vec<_>>());

        async move {
            async {
                let mut sum = Currency::default();
                let transactions = transactions_future.await?;
                for transaction in transactions {
                    match bill
                        .transactions
                        .get(&transaction.id)
                        .context(format!("Could not find transaction {}", transaction.id))?
                    {
                        Sign::Positive => sum += transaction.amount(),
                        Sign::Negative => sum -= transaction.amount(),
                    }
                }
                Ok::<_, anyhow::Error>(sum)
            }
            .await
            .context(format!(
                "Error while getting bill sum {} {}",
                bill.id, bill.name
            ))
        }
    }

    pub async fn get_bills(&self) -> Result<Vec<Bill>> {
        self.finance_manager
            .lock()
            .await
            .get_bills()
            .await
            .context("Error while getting bills")
    }

    pub async fn get_bill<'a>(&'a self, id: &'a Id) -> Result<Option<Bill>> {
        self.finance_manager
            .lock()
            .await
            .get_bill(id)
            .await
            .context(format!("Error while getting bill {}", id))
    }

    pub async fn create_bill(
        &self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> Result<Bill> {
        let mut ids = Vec::with_capacity(transactions.len());
        for transaction in &transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        self.finance_manager
            .lock()
            .await
            .create_bill(name, description, value, transactions, due_date)
            .await
            .context("Error while creating bill")
    }

    pub async fn delete_bill(&self, id: Id) -> Result<()> {
        self.finance_manager
            .lock()
            .await
            .delete_bill(id)
            .await
            .context(format!("Error while deleting bill with id {}", id))
    }

    pub async fn update_bill(&self, bill: Bill) -> Result<()> {
        let mut ids = Vec::with_capacity(bill.transactions.len());
        for transaction in &bill.transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        let bill_id = bill.id;
        self.finance_manager
            .lock()
            .await
            .update_bill(bill)
            .await
            .context(format!("Error while updating bill with id {}", bill_id))
    }

    pub async fn get_filtered_transactions(
        &self,
        filter: transaction_filter::TransactionFilter,
    ) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_filtered_transactions(filter)
            .await
            .context("Error while getting transactions with applied filter")
    }

    pub async fn create_asset_account(
        &self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        self.finance_manager
            .lock()
            .await
            .create_asset_account(name, note, iban, bic, offset)
            .await
            .context("Error while creating asset account")
    }

    pub async fn update_asset_account(
        &self,
        account: account::AssetAccount,
    ) -> Result<account::AssetAccount> {
        let acc_id = account.id;
        self.finance_manager
            .lock()
            .await
            .update_asset_account(account)
            .await
            .context(format!("Error while updating asset account {}", acc_id))
    }

    /// Deletes an account.
    /// If `purge_transactions` is true all transactions related to this account are deleted.
    /// If `purge_transactions` is false and transactions related to this account exist an error is thrown.
    pub async fn delete_account(
        &self,
        id: Id,
        purge_transactions: bool,
    ) -> std::result::Result<(), DeleteAccountError> {
        // get related transactions
        let transactions = self
            .get_transactions_of_account(id, (None, None))
            .await
            .context("fetching transactions of account failed")?;

        // check if account is used in transactions and raise error if necessary
        if !purge_transactions && !transactions.is_empty() {
            return Err(DeleteAccountError::RelatedTransactionsExist);
        }

        // delete transactions
        for transaction in transactions {
            self.delete_transaction(transaction.id)
                .await
                .context("could not delete transaction")?;
        }

        // delete account
        self.finance_manager
            .lock()
            .await
            .delete_account(id)
            .await
            .context("underlying delete account call on finance manager failed")?;

        Ok(())
    }

    pub async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        self.finance_manager
            .lock()
            .await
            .get_accounts()
            .await
            .context("Error while getting accounts")
    }

    pub async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        self.finance_manager
            .lock()
            .await
            .get_account(id)
            .await
            .context(format!("Error while deleting account with id {}", id))
    }

    pub async fn get_account_sum<'a>(
        &'a self,
        account: &'a account::Account,
        date: DateTime,
    ) -> Result<Currency> {
        let sum = self
            .finance_manager
            .lock()
            .await
            .get_account_sum(account, date)
            .await
            .context("Error while getting account sum")?;
        if let account::Account::AssetAccount(asset_account) = account {
            Ok(sum + &asset_account.offset)
        } else {
            Ok(sum)
        }
    }

    pub async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transaction(id)
            .await
            .context(format!("Error while getting transaction with id {}", id))
    }

    pub async fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transactions_of_account(account, timespan)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_transaction(
        &self,
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
        async {
            if amount.get_eur_num() < 0.0 {
                anyhow::bail!("Amount must be positive")
            }

            for category in &categories {
                if self.get_category(*category.0).await.unwrap().is_none() {
                    anyhow::bail!("Category does not exist!")
                }
            }

            self.finance_manager
                .lock()
                .await
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
        .await
        .context("Error while creating transaction")
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_transaction(&self, transaction: Transaction) -> Result<Transaction> {
        let t_id = transaction.id;
        async {
            if transaction.amount().get_eur_num() < 0.0 {
                anyhow::bail!("Amount must be positive")
            }
            self.finance_manager
                .lock()
                .await
                .update_transaction(transaction)
                .await
        }
        .await
        .context(format!("Error while updating transaction with id {}", t_id))
    }

    pub async fn create_book_checking_account(
        &self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
    ) -> Result<account::BookCheckingAccount> {
        self.finance_manager
            .lock()
            .await
            .create_book_checking_account(name, notes, iban, bic)
            .await
            .context("Error while creating book checking account")
    }

    pub async fn update_book_checking_account(
        &self,
        account: account::BookCheckingAccount,
    ) -> Result<account::BookCheckingAccount> {
        let acc_id = account.id;
        self.finance_manager
            .lock()
            .await
            .update_book_checking_account(account)
            .await
            .context(format!(
                "Error while updating book checking account with id {}",
                acc_id
            ))
    }

    pub async fn create_budget(
        &self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: budget::Recurring,
    ) -> Result<Budget> {
        self.finance_manager
            .lock()
            .await
            .create_budget(name, description, total_value, timespan)
            .await
            .context("Error while creating budget")
    }

    pub async fn delete_budget(&self, id: Id) -> Result<()> {
        self.finance_manager
            .lock()
            .await
            .delete_budget(id)
            .await
            .context(format!("Error while deleting budget with id {}", id))
    }

    pub async fn update_budget(&self, budget: Budget) -> Result<Budget> {
        let budget_id = budget.id;
        self.finance_manager
            .lock()
            .await
            .update_budget(budget)
            .await
            .context(format!("Error while updating budget with id {}", budget_id))
    }

    pub async fn get_budgets(&self) -> Result<Vec<Budget>> {
        self.finance_manager
            .lock()
            .await
            .get_budgets()
            .await
            .context("Error while getting budgets")
    }

    pub async fn get_budget(&self, id: Id) -> Result<Option<Budget>> {
        self.finance_manager
            .lock()
            .await
            .get_budget(id)
            .await
            .context(format!("Error while getting budget with id {}", id))
    }

    pub async fn delete_transaction(&self, id: Id) -> Result<()> {
        async move {
            for mut bill in self.get_bills().await? {
                if bill.transactions.remove(&id).is_some() {
                    self.update_bill(bill).await?
                }
            }
            self.finance_manager
                .lock()
                .await
                .delete_transaction(id)
                .await
                .context("underlying finance manager error")
        }
        .await
        .context(format!("Error while deleting transaction with id {}", id))
    }

    pub async fn get_transactions_in_timespan(
        &self,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transactions_in_timespan(timespan)
            .await
            .context("Error while getting transactions filtered by timespan")
    }

    pub async fn get_transactions(&self, ids: Vec<Id>) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transactions(ids)
            .await
            .context("Error while getting transactions")
    }

    pub async fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transactions_of_budget(id, timespan)
            .await
            .context(format!(
                "Error while getting transactions of budget with id {}",
                id
            ))
    }

    /// Gets the transactions of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub async fn get_budget_transactions<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
        timezone: time::UtcOffset,
    ) -> Result<Vec<Transaction>> {
        let timespan = budget::calculate_budget_timespan(
            budget,
            offset,
            time::OffsetDateTime::now_utc().to_offset(timezone),
        )?;
        self.get_transactions_of_budget(budget.id, timespan)
            .await
            .context(format!(
                "Error while getting transactions of budget with {} {}",
                budget.id, budget.name
            ))
    }

    /// Gets the value of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub async fn get_budget_value<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
        timezone: time::UtcOffset,
    ) -> Result<Currency> {
        let transactions = self
            .get_budget_transactions(budget, offset, timezone)
            .await
            .context(format!(
                "Error while getting value of budget {} {}",
                budget.id, budget.name
            ))?;
        let mut sum = Currency::default();
        for transaction in transactions {
            let sign = transaction.budget.unwrap().1;
            match sign {
                Sign::Positive => sum += transaction.amount(),
                Sign::Negative => sum -= transaction.amount(),
            }
        }
        Ok(sum)
    }

    pub fn get_accounts_hash_map(
        &self,
    ) -> impl Future<Output = Result<HashMap<Id, account::Account>>> + MaybeSend + '_ {
        let accounts_future = self.get_accounts();
        async {
            let accounts = accounts_future.await?;
            let mut account_map = HashMap::with_capacity(accounts.len());
            for account in accounts {
                account_map.insert(*account.id(), account);
            }
            Ok(account_map)
        }
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        self.finance_manager
            .lock()
            .await
            .get_categories()
            .await
            .context("Error while getting categories")
    }

    pub async fn get_category(&self, id: Id) -> Result<Option<Category>> {
        self.finance_manager
            .lock()
            .await
            .get_category(id)
            .await
            .context(format!("Error while getting category with id {}", id))
    }

    pub async fn create_category(&self, name: String) -> Result<Category> {
        self.finance_manager
            .lock()
            .await
            .create_category(name)
            .await
            .context("Error while creating category")
    }

    pub async fn update_category(&self, category: Category) -> Result<Category> {
        let category_id = category.id;
        self.finance_manager
            .lock()
            .await
            .update_category(category)
            .await
            .context(format!(
                "Error while updating category with id {}",
                category_id
            ))
    }

    pub async fn delete_category(&self, id: Id) -> Result<()> {
        self.finance_manager
            .lock()
            .await
            .delete_category(id)
            .await
            .context(format!("Error while deleting category with id {}", id))
    }

    pub async fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        self.finance_manager
            .lock()
            .await
            .get_transactions_of_category(id, timespan)
            .await
            .context(format!(
                "Error while getting transactions of category with id {} in timespan {:?}",
                id, timespan
            ))
    }

    /// Gets the values of the category over time.
    /// The first value is the value at the start of the timespan.
    /// The last value is the total value over the timespan.
    pub async fn get_relative_category_values(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<(DateTime, Currency)>> {
        Ok(sum_up_transactions_by_day(
            self.get_transactions_of_category(id, timespan)
                .await
                .context(format!(
                    "Error while getting transactions of category with id {} in timespan {:?}",
                    id, timespan
                ))?,
            |transaction| {
                *transaction
                    .categories
                    .clone()
                    .iter()
                    .find(|(x, _)| **x == id)
                    .unwrap()
                    .1
            },
        ))
    }

    pub async fn update_transaction_categories(
        &self,
        id: Id,
        categories: HashMap<Id, Sign>,
    ) -> Result<Transaction> {
        let mut transaction = self
            .get_transaction(id)
            .await
            .context(format!(
                "Error while updating categories for transaction with id {}",
                id
            ))?
            .unwrap();
        transaction.categories = categories;
        self.update_transaction(transaction).await.context(format!(
            "Error while updating categories for transaction with id {}",
            id
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteAccountError {
    #[error("Account is still used in transactions")]
    RelatedTransactionsExist,
    #[error("An error occurred: {0}")]
    Other(#[from] anyhow::Error),
}

#[cfg(test)]
mod test {
    use managers::RamFinanceManager;

    use super::*;
    use time::macros::*;

    #[async_std::test]
    async fn create_transaction_category_does_not_exist() {
        let fm = FMController::with_finance_manager(RamFinanceManager::new(()).unwrap());
        let acc1 = fm
            .create_asset_account(
                "asset_acc".to_string(),
                None,
                None,
                None,
                Currency::default(),
            )
            .await
            .unwrap();
        let acc2 = fm
            .create_book_checking_account("book_checking_acc".to_string(), None, None, None)
            .await
            .unwrap();
        assert!(
            fm.create_transaction(
                Currency::default(),
                "test".to_string(),
                None,
                acc1.id,
                acc2.id,
                None,
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(10:50)),
                HashMap::default(),
                HashMap::from([(1, Sign::Positive)]),
            )
            .await
            .is_err()
        )
    }

    #[async_std::test]
    async fn delete_transaction_in_bill_test() {
        let fm = FMController::with_finance_manager(RamFinanceManager::new(()).unwrap());
        let acc1 = fm
            .create_asset_account(
                "asset_acc".to_string(),
                None,
                None,
                None,
                Currency::default(),
            )
            .await
            .unwrap();
        let acc2 = fm
            .create_book_checking_account("book_checking_acc".to_string(), None, None, None)
            .await
            .unwrap();
        let t1 = fm
            .create_transaction(
                Currency::default(),
                "t1".to_string(),
                None,
                acc1.id,
                acc2.id,
                None,
                time::OffsetDateTime::now_utc(),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let bill = fm
            .create_bill(
                "test".to_string(),
                None,
                Currency::default(),
                HashMap::from([(t1.id, Sign::Positive)]),
                None,
            )
            .await
            .unwrap();
        fm.delete_transaction(t1.id).await.unwrap();
        let new_bill = fm.get_bill(&bill.id).await.unwrap().unwrap();
        assert!(new_bill.transactions.is_empty());
    }
}
