use super::{
    account, Budget, Currency, DateTime, FinanceManager, Id, Recouring, Timespan, Transaction,
};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RamFinanceManager {
    accounts: HashMap<Id, account::Account>,
    transactions: Vec<Transaction>,
    budgets: HashMap<Id, Budget>,
}

impl RamFinanceManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: Vec::new(),
            budgets: HashMap::new(),
        }
    }
}

impl FinanceManager for RamFinanceManager {
    async fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::AssetAccount> {
        let account = self.accounts.get_mut(&id).unwrap();
        let new_account = account::AssetAccount::new(id, name, note, iban, bic);
        *account = new_account.clone().into();
        Ok(new_account)
    }

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::AssetAccount> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_account = account::AssetAccount::new(id, name, note, iban, bic);

        if self.accounts.contains_key(&id) {
            panic!("ID ALREADY EXISTS");
        }

        self.accounts.insert(id, new_account.clone().into());

        Ok(new_account)
    }

    async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        return Ok(self
            .accounts
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<account::Account>>());
    }

    async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        if let Some(acc) = self.accounts.get(&id) {
            return Ok(Some(acc.clone()));
        }
        Ok(None)
    }

    async fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> Result<Currency> {
        // sum up all transactions from start to end date
        let transactions = self
            .get_transactions_of_account(account.id(), (None, Some(date)))
            .await?;
        let mut total = Currency::Eur(0.0);
        for transaction in transactions {
            if *transaction.source() == account.id() {
                total -= transaction.amount;
            } else {
                total += transaction.amount;
            }
        }
        Ok(total)
    }

    async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        for transaction in &self.transactions {
            if transaction.id == id {
                return Ok(Some(transaction.clone()));
            }
        }
        Ok(None)
    }

    async fn get_transactions_of_account(
        &self,
        account_id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        Ok(self
            .transactions
            .iter()
            .filter(|transaction| {
                if !transaction.connection_with_account(account_id) {
                    return false;
                }
                if let Some(begin) = timespan.0 {
                    if transaction.date < begin {
                        return false;
                    }
                }
                if let Some(end) = timespan.1 {
                    if transaction.date > end {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect())
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> Result<Budget> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_budget = Budget {
            id,
            name,
            description,
            total_value,
            timespan,
        };

        if self.budgets.contains_key(&id) {
            panic!("ID ALREADY EXISTS");
        }

        self.budgets.insert(id, new_budget.clone());

        Ok(new_budget)
    }

    async fn get_budgets(&self) -> Result<Vec<Budget>> {
        Ok(self
            .budgets
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<Budget>>())
    }

    async fn get_transactions_of_budget(&self, budget: &Budget) -> Result<Vec<Transaction>> {
        Ok(self
            .transactions
            .iter()
            .filter(|x| {
                if let Some(b) = x.budget {
                    if b == budget.id {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .collect())
    }

    async fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: super::Or<Id, String>,
        destination: super::Or<Id, String>,
        budget: Option<Id>,
        date: DateTime,
    ) -> Result<Transaction> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let source_id = match source {
            super::Or::One(id) => id,
            super::Or::Two(name) => {
                let account = self
                    .create_book_checking_account(name, None, None, None)
                    .await?;
                account.id()
            }
        };

        let destination_id = match destination {
            super::Or::One(id) => id,
            super::Or::Two(name) => {
                let account = self
                    .create_book_checking_account(name, None, None, None)
                    .await?;
                account.id()
            }
        };

        let new_transaction = Transaction {
            id,
            amount,
            title,
            description,
            source: source_id,
            destination: destination_id,
            budget,
            date,
        };

        self.transactions.push(new_transaction.clone());

        Ok(new_transaction)
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<account::BookCheckingAccount> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_account = account::BookCheckingAccount::new(id, name, notes, iban, bic);

        if self.accounts.contains_key(&id) {
            panic!("ID ALREADY EXISTS");
        }

        self.accounts.insert(id, new_account.clone().into());

        Ok(new_account)
    }

    async fn get_budget(&self, id: Id) -> Result<Option<Budget>> {
        match self.budgets.get(&id) {
            Some(x) => Ok(Some(x.clone())),
            None => Ok(None),
        }
    }
}
