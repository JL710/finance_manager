use crate::{
    account, AccountId, Bill, Budget, Category, Currency, DateTime, FinanceManager, Id, Recurring,
    Sign, Timespan, Transaction,
};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RamFinanceManager {
    accounts: HashMap<Id, account::Account>,
    transactions: Vec<Transaction>,
    budgets: HashMap<Id, Budget>,
    categories: Vec<Category>,
    bills: Vec<Bill>,
}

impl FinanceManager for RamFinanceManager {
    type Flags = ();

    fn new(_flags: Self::Flags) -> Result<Self> {
        Ok(Self::default())
    }

    async fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        let account = self.accounts.get_mut(&id).unwrap();
        let new_account = account::AssetAccount::new(id, name, note, iban, bic, offset);
        *account = new_account.clone().into();
        Ok(new_account)
    }

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
        offset: Currency,
    ) -> Result<account::AssetAccount> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_account = account::AssetAccount::new(id, name, note, iban, bic, offset);

        if self.accounts.contains_key(&id) {
            panic!("ID ALREADY EXISTS");
        }

        self.accounts.insert(id, new_account.clone().into());

        Ok(new_account)
    }

    async fn delete_account(&mut self, id: Id) -> Result<()> {
        self.accounts.remove(&id);
        Ok(())
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
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

    async fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        notes: Option<String>,
        iban: Option<AccountId>,
        bic: Option<String>,
    ) -> Result<account::BookCheckingAccount> {
        let account = self.accounts.get_mut(&id).unwrap();
        let new_account = account::BookCheckingAccount::new(id, name, notes, iban, bic);
        *account = new_account.clone().into();
        Ok(new_account)
    }

    async fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> Result<Bill> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_bill = Bill::new(id, name, description, value, transactions, due_date);

        self.bills.push(new_bill.clone());
        Ok(new_bill)
    }

    async fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
        due_date: Option<DateTime>,
    ) -> Result<()> {
        for bill in &mut self.bills {
            if bill.id == id {
                bill.name = name;
                bill.description = description;
                bill.value = value;
                bill.transactions = transactions;
                bill.due_date = due_date;
                return Ok(());
            }
        }
        Ok(())
    }

    async fn get_bills(&self) -> Result<Vec<Bill>> {
        Ok(self.bills.clone())
    }

    async fn get_bill(&self, id: &Id) -> Result<Option<Bill>> {
        for bill in &self.bills {
            if bill.id == *id {
                return Ok(Some(bill.clone()));
            }
        }
        Ok(None)
    }

    async fn delete_bill(&mut self, id: Id) -> Result<()> {
        self.bills.retain(|x| x.id != id);
        Ok(())
    }

    async fn get_accounts(&self) -> Result<Vec<account::Account>> {
        Ok(self
            .accounts
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<account::Account>>())
    }

    async fn get_account(&self, id: Id) -> Result<Option<account::Account>> {
        if let Some(acc) = self.accounts.get(&id) {
            return Ok(Some(acc.clone()));
        }
        Ok(None)
    }

    async fn get_transaction(&self, id: Id) -> Result<Option<Transaction>> {
        for transaction in &self.transactions {
            if *transaction.id() == id {
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
                    if transaction.date() < &begin {
                        return false;
                    }
                }
                if let Some(end) = timespan.1 {
                    if transaction.date() > &end {
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
        timespan: Recurring,
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

    async fn delete_budget(&mut self, id: Id) -> Result<()> {
        for transaction in &mut self.transactions {
            *transaction = Transaction::new(
                *transaction.id(),
                transaction.amount(),
                transaction.title().to_owned(),
                transaction.description().map(|x| x.to_owned()),
                *transaction.source(),
                *transaction.destination(),
                None,
                *transaction.date(),
                transaction.metadata().clone(),
                transaction.categories().clone(),
            )
        }
        self.budgets.remove(&id);
        Ok(())
    }

    async fn get_budgets(&self) -> Result<Vec<Budget>> {
        Ok(self
            .budgets
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<Budget>>())
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
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_transaction = Transaction::new(
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
        );

        self.transactions.push(new_transaction.clone());

        Ok(new_transaction)
    }

    async fn get_budget(&self, id: Id) -> Result<Option<Budget>> {
        match self.budgets.get(&id) {
            Some(x) => Ok(Some(x.clone())),
            None => Ok(None),
        }
    }

    async fn update_transaction(
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
    ) -> Result<Transaction> {
        let new_transaction = Transaction::new(
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
        );
        for transaction in &mut self.transactions {
            if *transaction.id() == id {
                *transaction = new_transaction.clone();
                return Ok(new_transaction);
            }
        }
        anyhow::bail!("Transaction does not exist");
    }

    async fn delete_transaction(&mut self, id: Id) -> Result<()> {
        let mut found_index = -1;
        for (index, transaction) in self.transactions.iter().enumerate() {
            if *transaction.id() == id {
                found_index = index as isize;
                break;
            }
        }
        if found_index == -1 {
            anyhow::bail!("Transaction does not exist");
        }
        self.transactions.remove(found_index as usize);
        Ok(())
    }

    async fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        Ok(self
            .transactions
            .iter()
            .filter(|transaction| {
                if let Some(budget_id) = transaction.budget() {
                    if budget_id.0 != id {
                        return false;
                    }
                } else {
                    return false;
                }
                if let Some(begin) = timespan.0 {
                    if transaction.date() < &begin {
                        return false;
                    }
                }
                if let Some(end) = timespan.1 {
                    if transaction.date() > &end {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect())
    }

    async fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
    ) -> Result<Budget> {
        let new_budget = Budget {
            id,
            name,
            description,
            total_value,
            timespan,
        };
        let old_budget = self.budgets.get_mut(&id).unwrap();
        *old_budget = new_budget.clone();
        Ok(new_budget)
    }

    async fn get_transactions_in_timespan(&self, timespan: Timespan) -> Result<Vec<Transaction>> {
        let mut transactions = self.transactions.clone();

        if let Some(begin) = timespan.0 {
            transactions.retain(|transaction| transaction.date() >= &begin);
        }

        if let Some(end) = timespan.1 {
            transactions.retain(|transaction| transaction.date() <= &end);
        }

        Ok(transactions)
    }

    async fn create_category(&mut self, name: String) -> Result<Category> {
        let id = uuid::Uuid::new_v4().as_u64_pair().0;

        let new_category = Category { id, name };

        self.categories.push(new_category.clone());

        Ok(new_category)
    }

    async fn update_category(&mut self, id: Id, name: String) -> Result<Category> {
        let new_category = Category { id, name };

        for category in &mut self.categories {
            if category.id == id {
                *category = new_category.clone();
                return Ok(new_category);
            }
        }

        anyhow::bail!("Category does not exist");
    }

    async fn get_categories(&self) -> Result<Vec<Category>> {
        Ok(self.categories.clone())
    }

    async fn get_category(&self, id: Id) -> Result<Option<Category>> {
        for category in &self.categories {
            if category.id == id {
                return Ok(Some(category.clone()));
            }
        }
        Ok(None)
    }

    async fn delete_category(&mut self, id: Id) -> Result<()> {
        let mut found_index = -1;
        for (index, category) in self.categories.iter().enumerate() {
            if category.id == id {
                found_index = index as isize;
                break;
            }
        }
        if found_index == -1 {
            anyhow::bail!("Category does not exist");
        }
        self.categories.remove(found_index as usize);

        // remove from transactions
        for transaction in &mut self.transactions {
            let mut categories = transaction.categories().clone();
            categories.retain(|x, _| *x != id);
            *transaction = Transaction::new(
                *transaction.id(),
                transaction.amount().clone(),
                transaction.title().clone(),
                transaction.description().map(|x| x.to_owned()),
                *transaction.source(),
                *transaction.destination(),
                transaction.budget().copied(),
                *transaction.date(),
                transaction.metadata().clone(),
                categories,
            );
        }

        Ok(())
    }

    async fn get_transactions_of_category(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> Result<Vec<Transaction>> {
        let mut transactions = self.transactions.clone();
        transactions.retain(|x| {
            x.categories()
                .iter()
                .map(|x| *x.0)
                .collect::<Vec<Id>>()
                .contains(&id)
        });

        if let Some(begin) = timespan.0 {
            transactions.retain(|transaction| transaction.date() >= &begin);
        }

        if let Some(end) = timespan.1 {
            transactions.retain(|transaction| transaction.date() <= &end);
        }

        Ok(transactions)
    }
}

crate::finance_manager_test::unit_tests!(|| async { super::RamFinanceManager::default() });
