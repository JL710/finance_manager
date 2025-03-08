use crate::FinanceManager;
use crate::*;
use anyhow::{Context, Result};
use std::future::Future;

#[derive(Clone, Debug)]
pub struct FMController<FM>
where
    FM: FinanceManager + 'static,
{
    finance_manager: FM,
}

impl<FM> FMController<FM>
where
    FM: FinanceManager,
{
    pub fn new(flags: FM::Flags) -> Result<Self> {
        Ok(Self {
            finance_manager: FM::new(flags)?,
        })
    }

    pub fn with_finance_manager(finance_manager: FM) -> Self {
        Self { finance_manager }
    }

    pub fn raw_fm(&self) -> &FM {
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
                        .get(transaction.id())
                        .context(format!("Could not find transaction {}", transaction.id()))?
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
                bill.id(),
                bill.name()
            ))
        }
    }

    pub fn get_bills(&self) -> impl Future<Output = Result<Vec<Bill>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_bills();
        async { fut.await.context("Error while getting bills") }
    }

    pub fn get_bill<'a>(
        &'a self,
        id: &'a Id,
    ) -> impl Future<Output = Result<Option<Bill>>> + MaybeSend + 'a {
        let fut = self.finance_manager.get_bill(id);
        async move {
            fut.await
                .context(format!("Error while getting bill {}", id))
        }
    }

    pub fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> Result<impl Future<Output = Result<Bill>> + MaybeSend + '_> {
        let mut ids = Vec::with_capacity(transactions.len());
        for transaction in &transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        Ok(async move {
            self.finance_manager
                .create_bill(name, description, value, transactions, due_date)
                .await
                .context("Error while creating bill")
        })
    }

    pub async fn delete_bill(&mut self, id: Id) -> Result<()> {
        self.finance_manager
            .delete_bill(id)
            .await
            .context(format!("Error while deleting bill with id {}", id))
    }

    pub fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> Result<impl Future<Output = Result<()>> + MaybeSend + '_> {
        let mut ids = Vec::with_capacity(transactions.len());
        for transaction in &transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        Ok(async move {
            self.finance_manager
                .update_bill(id, name, description, value, transactions, due_date)
                .await
                .context(format!("Error while updating bill with id {}", id))
        })
    }

    pub fn get_filtered_transactions(
        &self,
        filter: transaction_filter::TransactionFilter,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_filtered_transactions(filter);
        async move {
            fut.await
                .context("Error while getting transactions with applied filter")
        }
    }

    pub async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        self.finance_manager
            .create_asset_account(name, note, iban, make_iban_bic_unified(bic), offset)
            .await
            .context("Error while creating asset account")
    }

    pub async fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        self.finance_manager
            .update_asset_account(id, name, note, iban, make_iban_bic_unified(bic), offset)
            .await
            .context(format!("Error while updating asset account {}", id))
    }

    /// Deletes an account.
    /// If `purge_transactions` is true all transactions related to this account are deleted.
    /// If `purge_transactions` is false and transactions related to this account exist an error is thrown.
    pub async fn delete_account(
        &mut self,
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
            self.delete_transaction(*transaction.id())
                .await
                .context("could not delete transaction")?;
        }

        // delete account
        self.finance_manager
            .delete_account(id)
            .await
            .context("underlying delete account call on finance manager failed")?;

        Ok(())
    }

    pub fn get_accounts(
        &self,
    ) -> impl Future<Output = Result<Vec<account::Account>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_accounts();
        async { fut.await.context("Error while getting accounts") }
    }

    pub fn get_account(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<account::Account>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_account(id);
        async move {
            fut.await
                .context(format!("Error while deleting account with id {}", id))
        }
    }

    pub fn get_account_sum<'a>(
        &'a self,
        account: &'a account::Account,
        date: DateTime,
    ) -> impl Future<Output = Result<Currency>> + MaybeSend + 'a {
        let future = self.finance_manager.get_account_sum(account, date);

        async move {
            let sum = future.await.context("Error while getting account sum")?;
            if let account::Account::AssetAccount(asset_account) = account {
                Ok(sum + asset_account.offset().clone())
            } else {
                Ok(sum)
            }
        }
    }

    pub fn get_transaction(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<Transaction>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_transaction(id);
        async move {
            fut.await
                .context(format!("Error while getting transaction with id {}", id))
        }
    }

    pub fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        self.finance_manager
            .get_transactions_of_account(account, timespan)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_transaction(
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
    pub fn update_transaction(
        &mut self,
        id: Id,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Id,
        destination: Id,
        budget: Option<(Id, Sign)>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categories: HashMap<Id, Sign>,
    ) -> Result<impl Future<Output = Result<Transaction>> + MaybeSend + '_> {
        if amount.get_eur_num() < 0.0 {
            anyhow::bail!("Amount must be positive")
        }
        Ok(async move {
            self.finance_manager
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
                .context(format!("Error while updating transaction with id {}", id))
        })
    }

    pub fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl Future<Output = Result<account::BookCheckingAccount>> + MaybeSend + '_ {
        let fut = self.finance_manager.create_book_checking_account(
            name,
            notes,
            iban,
            make_iban_bic_unified(bic),
        );
        async move {
            fut.await
                .context("Error while creating book checking account")
        }
    }

    pub fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl Future<Output = Result<account::BookCheckingAccount>> + MaybeSend + '_ {
        let fut = self.finance_manager.update_book_checking_account(
            id,
            name,
            note,
            iban,
            make_iban_bic_unified(bic),
        );
        async move {
            fut.await.context(format!(
                "Error while updating book checking account with id {}",
                id
            ))
        }
    }

    pub fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
    ) -> impl Future<Output = Result<Budget>> + MaybeSend + '_ {
        let fut = self
            .finance_manager
            .create_budget(name, description, total_value, timespan);
        async move { fut.await.context("Error while creating budget") }
    }

    pub fn delete_budget(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend + '_ {
        let fut = self.finance_manager.delete_budget(id);
        async move {
            fut.await
                .context(format!("Error while deleting budget with id {}", id))
        }
    }

    pub fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
    ) -> impl Future<Output = Result<Budget>> + MaybeSend + '_ {
        let fut = self
            .finance_manager
            .update_budget(id, name, description, total_value, timespan);
        async move {
            fut.await
                .context(format!("Error while updating budget with id {}", id))
        }
    }

    pub fn get_budgets(&self) -> impl Future<Output = Result<Vec<Budget>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_budgets();
        async { fut.await.context("Error while getting budgets") }
    }

    pub fn get_budget(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<Budget>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_budget(id);
        async move {
            fut.await
                .context(format!("Error while getting budget with id {}", id))
        }
    }

    pub async fn delete_transaction(&mut self, id: Id) -> Result<()> {
        async move {
            for bill in self.get_bills().await? {
                if bill.transactions().iter().any(|(x, _)| *x == id) {
                    self.update_bill(
                        *bill.id(),
                        bill.name().clone(),
                        bill.description().to_owned(),
                        bill.value().to_owned(),
                        bill.transactions().to_owned(),
                        bill.due_date().to_owned(),
                    )?
                    .await?;
                }
            }
            self.finance_manager
                .delete_transaction(id)
                .await
                .context("underlying finance manager error")
        }
        .await
        .context(format!("Error while deleting transaction with id {}", id))
    }

    pub fn get_transactions_in_timespan(
        &self,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_transactions_in_timespan(timespan);
        async {
            fut.await
                .context("Error while getting transactions filtered by timespan")
        }
    }

    pub fn get_transactions(
        &self,
        ids: Vec<Id>,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_transactions(ids);
        async { fut.await.context("Error while getting transactions") }
    }

    pub fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let fut = self
            .finance_manager
            .get_transactions_of_budget(id, timespan);
        async move {
            fut.await.context(format!(
                "Error while getting transactions of budget with id {}",
                id
            ))
        }
    }

    /// Gets the transactions of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub fn get_budget_transactions<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
        timezone: time::UtcOffset,
    ) -> Result<impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + 'a> {
        let timespan = calculate_budget_timespan(
            budget,
            offset,
            time::OffsetDateTime::now_utc().to_offset(timezone),
        )?;
        let fut = self.get_transactions_of_budget(*budget.id(), timespan);
        Ok(async move {
            fut.await.context(format!(
                "Error while getting transactions of budget with {} {}",
                budget.id(),
                budget.name()
            ))
        })
    }

    /// Gets the value of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub fn get_budget_value<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
        timezone: time::UtcOffset,
    ) -> Result<impl Future<Output = Result<Currency>> + MaybeSend + 'a> {
        let transactions_future = self.get_budget_transactions(budget, offset, timezone)?;
        Ok(async move {
            let transactions = transactions_future.await.context(format!(
                "Error while getting value of budget {} {}",
                budget.id(),
                budget.name()
            ))?;
            let mut sum = Currency::default();
            for transaction in transactions {
                let sign = transaction.budget().unwrap().1;
                match sign {
                    Sign::Positive => sum += transaction.amount(),
                    Sign::Negative => sum -= transaction.amount(),
                }
            }
            Ok(sum)
        })
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

    pub fn get_categories(&self) -> impl Future<Output = Result<Vec<Category>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_categories();
        async { fut.await.context("Error while getting categories") }
    }

    pub fn get_category(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<Category>>> + MaybeSend + '_ {
        let fut = self.finance_manager.get_category(id);
        async move {
            fut.await
                .context(format!("Error while getting category with id {}", id))
        }
    }

    pub fn create_category(
        &mut self,
        name: String,
    ) -> impl Future<Output = Result<Category>> + MaybeSend + '_ {
        let fut = self.finance_manager.create_category(name);
        async { fut.await.context("Error while creating category") }
    }

    pub fn update_category(
        &mut self,
        id: Id,
        name: String,
    ) -> impl Future<Output = Result<Category>> + MaybeSend + '_ {
        let fut = self.finance_manager.update_category(id, name);
        async move {
            fut.await
                .context(format!("Error while updating category with id {}", id))
        }
    }

    pub fn delete_category(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend + '_ {
        let fut = self.finance_manager.delete_category(id);
        async move {
            fut.await
                .context(format!("Error while deleting category with id {}", id))
        }
    }

    pub fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let fut = self
            .finance_manager
            .get_transactions_of_category(id, timespan);
        async move {
            fut.await.context(format!(
                "Error while getting transactions of category with id {} in timespan {:?}",
                id, timespan
            ))
        }
    }

    /// Gets the values of the category over time.
    /// The first value is the value at the start of the timespan.
    /// The last value is the total value over the timespan.
    pub fn get_relative_category_values(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<(DateTime, Currency)>>> + MaybeSend + '_ {
        let transactions_future = self.get_transactions_of_category(id, timespan);
        async move {
            Ok(sum_up_transactions_by_day(
                transactions_future.await.context(format!(
                    "Error while getting transactions of category with id {} in timespan {:?}",
                    id, timespan
                ))?,
                |transaction| {
                    *transaction
                        .categories()
                        .clone()
                        .iter()
                        .find(|(x, _)| **x == id)
                        .unwrap()
                        .1
                },
            ))
        }
    }

    pub async fn update_transaction_categories(
        &mut self,
        id: Id,
        categories: HashMap<Id, Sign>,
    ) -> Result<Transaction> {
        let transaction = self
            .get_transaction(id)
            .await
            .context(format!(
                "Error while updating categories for transaction with id {}",
                id
            ))?
            .unwrap();
        self.update_transaction(
            *transaction.id(),
            transaction.amount().clone(),
            transaction.title().to_owned(),
            transaction.description().map(|x| x.to_owned()),
            *transaction.source(),
            *transaction.destination(),
            transaction.budget().map(|x| x.to_owned()),
            *transaction.date(),
            transaction.metadata().clone(),
            categories,
        )?
        .await
        .context(format!(
            "Error while updating categories for transaction with id {}",
            id
        ))
    }
}

fn make_iban_bic_unified(content: Option<String>) -> Option<String> {
    content.map(|content| content.to_uppercase().replace(' ', "").trim().to_string())
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
        let mut fm = FMController::with_finance_manager(RamFinanceManager::new(()).unwrap());
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
                acc1.id(),
                acc2.id(),
                None,
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(10:50)),
                HashMap::default(),
                HashMap::from([(1, Sign::Positive)]),
            )
            .await
            .is_err()
        )
    }
}
