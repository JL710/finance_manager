use crate::FinanceManager;
use crate::*;

use anyhow::Result;

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
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend + 'a {
        let transactions = bill
            .transactions()
            .clone()
            .into_iter()
            .map(|(id, sign)| (self.finance_manager.get_transaction(id), sign))
            .collect::<Vec<_>>();

        async move {
            let mut sum = Currency::default();
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

    pub fn get_bills(&self) -> impl futures::Future<Output = Result<Vec<Bill>>> + MaybeSend + '_ {
        self.finance_manager.get_bills()
    }

    pub fn get_bill<'a>(
        &'a self,
        id: &'a Id,
    ) -> impl futures::Future<Output = Result<Option<Bill>>> + MaybeSend + 'a {
        self.finance_manager.get_bill(id)
    }

    pub fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> Result<impl futures::Future<Output = Result<Bill>> + MaybeSend + '_> {
        let mut ids = Vec::with_capacity(transactions.len());
        for transaction in &transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        Ok(self
            .finance_manager
            .create_bill(name, description, value, transactions, due_date))
    }

    pub fn delete_bill(
        &mut self,
        id: Id,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend + '_ {
        self.finance_manager.delete_bill(id)
    }

    pub fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> Result<impl futures::Future<Output = Result<()>> + MaybeSend + '_> {
        let mut ids = Vec::with_capacity(transactions.len());
        for transaction in &transactions {
            if ids.contains(&transaction.0) {
                anyhow::bail!("Bill cannot have a transaction twice")
            }
            ids.push(transaction.0);
        }
        Ok(self
            .finance_manager
            .update_bill(id, name, description, value, transactions, due_date))
    }

    pub fn get_filtered_transactions(
        &self,
        filter: transaction_filter::TransactionFilter,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        let transactions_future = self.get_transactions(filter.total_timespan());
        async move {
            let transactions = transactions_future.await?;
            Ok(filter.filter_transactions(transactions))
        }
    }

    pub fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend + '_ {
        self.finance_manager.create_asset_account(
            name,
            note,
            iban,
            make_iban_bic_unified(bic),
            offset,
        )
    }

    pub fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend + '_ {
        self.finance_manager.update_asset_account(
            id,
            name,
            note,
            iban,
            make_iban_bic_unified(bic),
            offset,
        )
    }

    pub fn get_accounts(
        &self,
    ) -> impl futures::Future<Output = Result<Vec<account::Account>>> + MaybeSend + '_ {
        self.finance_manager.get_accounts()
    }

    pub fn get_account(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<account::Account>>> + MaybeSend + '_ {
        self.finance_manager.get_account(id)
    }

    pub fn get_account_sum<'a>(
        &'a self,
        account: &'a account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend + 'a {
        let future = self.finance_manager.get_account_sum(account, date);

        async move {
            let sum = future.await?;
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
    ) -> impl futures::Future<Output = Result<Option<Transaction>>> + MaybeSend + '_ {
        self.finance_manager.get_transaction(id)
    }

    pub fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        self.finance_manager
            .get_transactions_of_account(account, timespan)
    }

    pub fn create_transaction(
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
    ) -> Result<impl futures::Future<Output = Result<Transaction>> + MaybeSend + '_> {
        if amount.get_eur_num() < 0.0 {
            anyhow::bail!("Amount must be positive")
        }
        Ok(self.finance_manager.create_transaction(
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
        categories: Vec<(Id, Sign)>,
    ) -> Result<impl futures::Future<Output = Result<Transaction>> + MaybeSend + '_> {
        if amount.get_eur_num() < 0.0 {
            anyhow::bail!("Amount must be positive")
        }
        Ok(self.finance_manager.update_transaction(
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
        ))
    }

    pub fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend + '_ {
        self.finance_manager.create_book_checking_account(
            name,
            notes,
            iban,
            make_iban_bic_unified(bic),
        )
    }

    pub fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend + '_ {
        self.finance_manager.update_book_checking_account(
            id,
            name,
            note,
            iban,
            make_iban_bic_unified(bic),
        )
    }

    pub fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + MaybeSend + '_ {
        self.finance_manager
            .create_budget(name, description, total_value, timespan)
    }

    pub fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + MaybeSend + '_ {
        self.finance_manager
            .update_budget(id, name, description, total_value, timespan)
    }

    pub fn get_budgets(
        &self,
    ) -> impl futures::Future<Output = Result<Vec<Budget>>> + MaybeSend + '_ {
        self.finance_manager.get_budgets()
    }

    pub fn get_budget(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Budget>>> + MaybeSend + '_ {
        self.finance_manager.get_budget(id)
    }

    pub fn delete_transaction(
        &mut self,
        id: Id,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend + '_ {
        self.finance_manager.delete_transaction(id)
    }

    pub fn get_transactions(
        &self,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        self.finance_manager.get_transactions(timespan)
    }

    pub fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        self.finance_manager
            .get_transactions_of_budget(id, timespan)
    }

    /// Gets the transactions of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub fn get_budget_transactions<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + 'a {
        let timespan = calculate_budget_timespan(budget, offset, chrono::Utc::now());
        self.get_transactions_of_budget(*budget.id(), timespan)
    }

    /// Gets the value of the budget at the current timespan if the offset is 0.
    ///
    /// If the offset is positive the timespan is in the future. If the offset is negative the timespan is in the past.
    pub fn get_budget_value<'a>(
        &'a self,
        budget: &'a Budget,
        offset: i32,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend + 'a {
        let transactions_future = self.get_budget_transactions(budget, offset);
        async {
            let transactions = transactions_future.await?;
            let mut sum = Currency::default();
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

    pub fn get_accounts_hash_map(
        &self,
    ) -> impl futures::Future<Output = Result<HashMap<Id, account::Account>>> + MaybeSend + '_ {
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

    pub fn get_categories(
        &self,
    ) -> impl futures::Future<Output = Result<Vec<Category>>> + MaybeSend + '_ {
        self.finance_manager.get_categories()
    }

    pub fn get_category(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Category>>> + MaybeSend + '_ {
        self.finance_manager.get_category(id)
    }

    pub fn create_category(
        &mut self,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + MaybeSend + '_ {
        self.finance_manager.create_category(name)
    }

    pub fn update_category(
        &mut self,
        id: Id,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + MaybeSend + '_ {
        self.finance_manager.update_category(id, name)
    }

    pub fn delete_category(
        &mut self,
        id: Id,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend + '_ {
        self.finance_manager.delete_category(id)
    }

    pub fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend + '_ {
        self.finance_manager
            .get_transactions_of_category(id, timespan)
    }

    /// Gets the values of the category over time.
    /// The first value is the value at the start of the timespan.
    /// The last value is the total value over the timespan.
    pub fn get_relative_category_values(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<(DateTime, Currency)>>> + MaybeSend + '_ {
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

fn make_iban_bic_unified(content: Option<String>) -> Option<String> {
    content.map(|content| content.to_uppercase().replace(' ', "").trim().to_string())
}
