use std::collections::HashMap;

pub mod account;

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Id = u128;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Currency {
    Eur(f64),
}

impl Currency {
    pub fn to_num_string(&self) -> String {
        match self {
            Currency::Eur(num) => num.to_string(),
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::Eur(value) => write!(f, "{}€", value),
        }
    }
}

impl std::ops::Add for Currency {
    type Output = Currency;

    fn add(self, other: Currency) -> Self::Output {
        match self {
            Currency::Eur(value) => match other {
                Currency::Eur(other_value) => Currency::Eur(value + other_value),
            },
        }
    }
}

impl std::ops::AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    id: Id,
    name: String,
    description: Option<String>,
    total_value: Currency,
    timespan: Recourung,
}

impl Budget {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(desc) => Some(desc),
            None => None,
        }
    }

    pub fn total_value(&self) -> Currency {
        self.total_value.clone()
    }

    pub fn timespan(&self) -> &Recourung {
        &self.timespan
    }
}

impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    id: Id,
    amount: Currency, // if amount is positive the money is added to the account. If negative it is removed
    title: String,
    description: Option<String>,
    source: Option<Id>,
    destination: Option<Id>,
    budget: Option<Id>,
    date: DateTime,
}

impl Transaction {
    fn connection_with_account(&self, account: &account::Account) -> bool {
        if let Some(source) = self.source {
            if *account == source {
                return true;
            }
        }
        if let Some(destination) = self.destination {
            if *account == destination {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct FinanceManager {
    accounts: HashMap<Id, account::Account>,
    transactions: Vec<Transaction>,
    budgets: HashMap<Id, Budget>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Recourung {
    Days(DateTime, usize), // start time and days
    DayInMonth(u16),       // i.e. 3. of each month
    Yearly(u8, u16),       // month and day
}

type Timespan = (Option<DateTime>, Option<DateTime>);

impl FinanceManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: Vec::new(),
            budgets: HashMap::new(),
        }
    }

    pub fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> account::AssetAccount {
        let id = uuid::Uuid::new_v4().as_u128();

        let new_account = account::AssetAccount::new(id, name, note, iban, bic);

        if self.accounts.contains_key(&id) {
            panic!("ID ALREADY EXISTS");
        }

        self.accounts.insert(id, new_account.clone().into());

        new_account
    }

    pub fn get_accounts(&self) -> Vec<account::Account> {
        return self
            .accounts
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<account::Account>>();
    }

    pub fn get_account(&self, id: Id) -> Option<account::Account> {
        if let Some(acc) = self.accounts.get(&id) {
            return Some(acc.clone());
        }
        None
    }

    pub fn get_account_sum(&self, account: &account::Account, date: DateTime) -> Currency {
        // sum up all transactions from start to end date
        let transactions = self.get_transactions_of_account(account, (None, Some(date)));
        let mut total = Currency::Eur(0.0);
        for transaction in transactions {
            total += transaction.amount;
        }
        total
    }

    pub fn get_transaction(&self, id: Id) -> Option<Transaction> {
        for transaction in &self.transactions {
            if transaction.id == id {
                return Some(transaction.clone());
            }
        }
        None
    }

    pub fn get_transactions_of_account(
        &self,
        account: &account::Account,
        timespan: Timespan,
    ) -> Vec<Transaction> {
        self.transactions
            .iter()
            .filter(|transaction| {
                if !transaction.connection_with_account(account) {
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
            .collect()
    }

    pub fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recourung,
    ) -> Budget {
        let id = uuid::Uuid::new_v4().as_u128();

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

        new_budget
    }

    pub fn get_budgets(&self) -> Vec<Budget> {
        self.budgets
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<Budget>>()
    }

    pub fn get_transactions_of_budget(&self, budget: &Budget) -> Vec<Transaction> {
        self.transactions
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
            .collect()
    }
}
