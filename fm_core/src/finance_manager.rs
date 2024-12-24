use super::*;
use std::future::Future;

pub trait FinanceManager: Send + Clone + Sized {
    type Flags;

    fn new(flags: Self::Flags) -> Result<Self>;

    fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl Future<Output = Result<account::AssetAccount>> + MaybeSend;

    fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl Future<Output = Result<account::AssetAccount>> + MaybeSend;

    /// This should only delete the account and nothing else (like asserted transactions).
    fn delete_account(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend;

    fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    /// Only get the sum of the transactions for the account at the given date.
    /// Do not include any AssetAccount.offset or similar!
    /// This should almost never be overwritten!
    fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl Future<Output = Result<Currency>> + MaybeSend {
        let transactions_future =
            self.get_transactions_of_account(*account.id(), (None, Some(date)));

        async move {
            let transactions = transactions_future.await?;

            let mut sum = Currency::default();
            for transaction in transactions {
                if transaction.source() == account.id() {
                    sum -= transaction.amount();
                } else if transaction.destination() == account.id() {
                    sum += transaction.amount();
                }
            }
            Ok(sum)
        }
    }

    fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> impl Future<Output = Result<Bill>> + MaybeSend;

    fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> impl Future<Output = Result<()>> + MaybeSend;

    fn get_bills(&self) -> impl Future<Output = Result<Vec<Bill>>> + MaybeSend;

    fn get_bill(&self, id: &Id) -> impl Future<Output = Result<Option<Bill>>> + MaybeSend;

    fn delete_bill(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend;

    fn get_filtered_transactions(
        &self,
        filter: transaction_filter::TransactionFilter,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend {
        let transactions_future = self.get_transactions_in_timespan(filter.total_timespan());
        async move {
            let transactions = transactions_future.await?;
            Ok(filter.filter_transactions(transactions))
        }
    }

    fn get_accounts(&self) -> impl Future<Output = Result<Vec<account::Account>>> + MaybeSend;

    fn get_account(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<account::Account>>> + MaybeSend;

    fn get_transaction(
        &self,
        id: Id,
    ) -> impl Future<Output = Result<Option<Transaction>>> + MaybeSend;

    fn get_transactions(
        &self,
        ids: Vec<Id>,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend {
        let mut futures = Vec::with_capacity(ids.len());
        for id in ids {
            futures.push(self.get_transaction(id));
        }
        async move {
            let mut transactions = Vec::with_capacity(futures.len());
            for future in futures {
                transactions.push(future.await?.unwrap());
            }
            Ok(transactions)
        }
    }

    fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    #[allow(clippy::too_many_arguments)]
    fn create_transaction(
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
    ) -> impl Future<Output = Result<Transaction>> + MaybeSend;

    #[allow(clippy::too_many_arguments)]
    fn update_transaction(
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
    ) -> impl Future<Output = Result<Transaction>> + MaybeSend;

    fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
    ) -> impl Future<Output = Result<Budget>> + MaybeSend;

    fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
    ) -> impl Future<Output = Result<Budget>> + MaybeSend;

    fn get_budgets(&self) -> impl Future<Output = Result<Vec<Budget>>> + MaybeSend;

    fn get_budget(&self, id: Id) -> impl Future<Output = Result<Option<Budget>>> + MaybeSend;

    /// This function should only delete the transaction it self.
    fn delete_transaction(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend;

    fn get_transactions_in_timespan(
        &self,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    fn get_accounts_hash_map(
        &self,
    ) -> impl Future<Output = Result<HashMap<Id, account::Account>>> + MaybeSend {
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

    fn get_categories(&self) -> impl Future<Output = Result<Vec<Category>>> + MaybeSend;

    fn get_category(&self, id: Id) -> impl Future<Output = Result<Option<Category>>> + MaybeSend;

    fn create_category(
        &mut self,
        name: String,
    ) -> impl Future<Output = Result<Category>> + MaybeSend;

    fn update_category(
        &mut self,
        id: Id,
        name: String,
    ) -> impl Future<Output = Result<Category>> + MaybeSend;

    // delete category and remove it from every transaction
    fn delete_category(&mut self, id: Id) -> impl Future<Output = Result<()>> + MaybeSend;

    fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    /// Gets the values of the category over time.
    /// The first value is the value at the start of the timespan.
    /// The last value is the total value over the timespan.
    fn get_relative_category_values(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl Future<Output = Result<Vec<(DateTime, Currency)>>> + MaybeSend {
        let transactions_future = self.get_transactions_of_category(id, timespan);
        async move {
            Ok(sum_up_transactions_by_day(
                transactions_future.await?,
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
}

#[allow(unused_macros)]
macro_rules! unit_tests {
    ($gen_fm:expr) => {
        #[cfg(test)]
        mod test {
            use super::*;
            use crate::FinanceManager;

            #[async_std::test]
            async fn create_asset_account() {
                let mut fm = ($gen_fm)();
                let account = fm
                    .create_asset_account("Test".to_string(), None, None, None, Currency::default())
                    .await
                    .unwrap();
                assert_eq!(account.name(), "Test");
                assert_eq!(account.note(), None);
                assert_eq!(*account.iban(), None);
                assert_eq!(account.bic(), None);
                assert_eq!(*account.offset(), Currency::default());

                if let account::Account::AssetAccount(fetched_account) =
                    fm.get_account(account.id()).await.unwrap().unwrap()
                {
                    assert_eq!(fetched_account, account);
                } else {
                    assert!(false);
                }
            }

            #[async_std::test]
            async fn get_accounts() {
                let mut fm = ($gen_fm)();
                let accounts = fm.get_accounts().await.unwrap();
                assert_eq!(accounts.len(), 0);

                let acc = fm
                    .create_asset_account("Test".to_string(), None, None, None, Currency::default())
                    .await
                    .unwrap();
                let accounts = fm.get_accounts().await.unwrap();
                assert_eq!(accounts.len(), 1);
                assert_eq!(accounts[0], account::Account::AssetAccount(acc));
            }

            #[async_std::test]
            async fn create_book_checking_account() {
                let mut fm = ($gen_fm)();
                let account = fm
                    .create_book_checking_account("Test".to_string(), None, None, None)
                    .await
                    .unwrap();
                assert_eq!(account.name(), "Test");
                assert_eq!(account.note(), None);
                assert_eq!(*account.iban(), None);
                assert_eq!(account.bic(), None);

                if let account::Account::BookCheckingAccount(fetched_account) =
                    fm.get_account(account.id()).await.unwrap().unwrap()
                {
                    assert_eq!(fetched_account, account);
                } else {
                    assert!(false);
                }
            }

            #[async_std::test]
            async fn delete_category() {
                let mut fm = ($gen_fm)();

                let acc1 = fm
                    .create_asset_account(
                        "Test1".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let acc2 = fm
                    .create_asset_account(
                        "Test2".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let category = fm.create_category("Test".to_string()).await.unwrap();

                let transaction = fm
                    .create_transaction(
                        Currency::default(),
                        "Test".to_string(),
                        None,
                        acc1.id(),
                        acc2.id(),
                        None,
                        DateTime::now_utc(),
                        HashMap::new(),
                        [(category.id().clone(), Sign::Positive)]
                            .iter()
                            .cloned()
                            .collect(),
                    )
                    .await
                    .unwrap();

                fm.delete_category(*category.id()).await.unwrap();

                // check if category is deleted
                assert!(fm.get_category(*category.id()).await.unwrap().is_none());

                // check if category is removed from transactions
                assert!(fm
                    .get_transaction(*transaction.id())
                    .await
                    .unwrap()
                    .unwrap()
                    .categories()
                    .is_empty());
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use unit_tests;
