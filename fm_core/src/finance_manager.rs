use super::*;

pub trait FinanceManager: Send + Clone + Sized {
    type Flags;

    fn new(flags: Self::Flags) -> Result<Self>;

    fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend;

    fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend;

    fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    /// Only get the sum of the transactions for the account at the given date.
    /// Do not include any AssetAccount.offset or similar!
    fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend;

    fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> impl futures::Future<Output = Result<Bill>> + MaybeSend;

    fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend;

    fn get_bill_sum(
        &self,
        bill: &Bill,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend {
        let transactions = bill
            .transactions()
            .clone()
            .into_iter()
            .map(|(id, sign)| (self.get_transaction(id), sign))
            .collect::<Vec<_>>();

        async move {
            let mut sum = Currency::Eur(0.0);
            for (transaction_future, sign) in transactions {
                let transaction = transaction_future.await?.unwrap();
                match sign {
                    Sign::Positive => sum += transaction.amount(),
                    Sign::Negative => sum -= transaction.amount(),
                }
            }
            Ok(sum)
        }
    }

    fn get_bills(&self) -> impl futures::Future<Output = Result<Vec<Bill>>> + MaybeSend;

    fn get_bill(&self, id: &Id) -> impl futures::Future<Output = Result<Option<Bill>>> + MaybeSend;

    fn delete_bill(&mut self, id: Id) -> impl futures::Future<Output = Result<()>> + MaybeSend;

    fn get_filtered_transactions(
        &self,
        filter: transaction_filter::TransactionFilter,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend {
        let transactions_future = self.get_transactions(filter.total_timespan());
        async move {
            let transactions = transactions_future.await?;
            Ok(filter.filter_transactions(transactions))
        }
    }

    fn get_accounts(
        &self,
    ) -> impl futures::Future<Output = Result<Vec<account::Account>>> + MaybeSend;

    fn get_account(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<account::Account>>> + MaybeSend;

    fn get_transaction(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Transaction>>> + MaybeSend;

    fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend;

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
        categories: Vec<(Id, Sign)>,
    ) -> impl futures::Future<Output = Result<Transaction>> + MaybeSend;

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
        categories: Vec<(Id, Sign)>,
    ) -> impl futures::Future<Output = Result<Transaction>> + MaybeSend;

    fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + MaybeSend;

    fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + MaybeSend;

    fn get_budgets(&self) -> impl futures::Future<Output = Result<Vec<Budget>>> + MaybeSend;

    fn get_budget(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Budget>>> + MaybeSend;

    fn delete_transaction(
        &mut self,
        id: Id,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend;

    fn get_transactions(
        &self,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    /// Gets the transactions of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    fn get_budget_transactions(
        &self,
        budget: &Budget,
        offset: i32,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend {
        let timespan = calculate_budget_timespan(budget, offset, chrono::Utc::now());
        self.get_transactions_of_budget(*budget.id(), timespan)
    }

    /// Gets the value of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    fn get_budget_value(
        &self,
        budget: &Budget,
        offset: i32,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend {
        let transactions_future = self.get_budget_transactions(budget, offset);
        async {
            let transactions = transactions_future.await?;
            let mut sum = Currency::Eur(0.0);
            for transaction in transactions {
                let sign = transaction.budget().unwrap().1;
                match sign {
                    Sign::Positive => sum += transaction.amount(),
                    Sign::Negative => sum -= transaction.amount(),
                }
            }
            Ok(sum)
        }
    }

    fn get_accounts_hash_map(
        &self,
    ) -> impl futures::Future<Output = Result<HashMap<Id, account::Account>>> + MaybeSend {
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

    fn get_categories(&self) -> impl futures::Future<Output = Result<Vec<Category>>> + MaybeSend;

    fn get_category(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Category>>> + MaybeSend;

    fn create_category(
        &mut self,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + MaybeSend;

    fn update_category(
        &mut self,
        id: Id,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + MaybeSend;

    fn delete_category(&mut self, id: Id) -> impl futures::Future<Output = Result<()>> + MaybeSend;

    fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend;

    /// Gets the values of the category over time.
    /// The first value is the value at the start of the timespan.
    /// The last value is the total value over the timespan.
    fn get_relative_category_values(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<(DateTime, Currency)>>> + MaybeSend {
        let transactions_future = self.get_transactions_of_category(id, timespan);
        async move {
            Ok(sum_up_transactions_by_day(
                transactions_future.await?,
                |transaction| {
                    transaction
                        .categories()
                        .clone()
                        .iter()
                        .find(|(x, _)| *x == id)
                        .unwrap()
                        .1
                },
            ))
        }
    }
}
