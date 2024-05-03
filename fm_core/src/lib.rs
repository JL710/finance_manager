use anyhow::Result;
use chrono::{Datelike, TimeZone};
use std::collections::HashMap;

pub mod account;
#[cfg(feature = "ram")]
pub mod ram_finance_manager;
#[cfg(feature = "sqlite")]
pub mod sqlite_finange_manager;

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Id = u64;

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

    pub fn get_num(&self) -> f64 {
        match self {
            Currency::Eur(x) => *x,
        }
    }

    pub fn get_currency_id(&self) -> i32 {
        match self {
            Currency::Eur(_) => 1,
        }
    }

    pub fn from_currency_id(id: i32, amound: f64) -> Result<Self> {
        match id {
            1 => Ok(Currency::Eur(amound)),
            _ => anyhow::bail!("not a valid currency id"),
        }
    }

    pub fn negative(&self) -> Self {
        match self {
            Currency::Eur(x) => Currency::Eur(-*x),
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

impl std::ops::Sub for Currency {
    type Output = Currency;

    fn sub(self, other: Currency) -> Self::Output {
        match self {
            Currency::Eur(value) => match other {
                Currency::Eur(other_value) => Currency::Eur(value - other_value),
            },
        }
    }
}

impl std::ops::AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl std::ops::SubAssign for Currency {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    id: Id,
    name: String,
    description: Option<String>,
    total_value: Currency,
    timespan: Recouring,
}

impl Budget {
    pub fn new(
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> Self {
        Self {
            id,
            name,
            description,
            total_value,
            timespan,
        }
    }

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

    pub fn timespan(&self) -> &Recouring {
        &self.timespan
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    id: Id,
    name: String,
}

impl Category {
    pub fn new(id: Id, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    id: Id,
    amount: Currency, // if amount is positive the money is added to the account. If negative it is removed
    title: String,
    description: Option<String>,
    source: Id,
    destination: Id,
    budget: Option<Id>,
    date: DateTime,
    metadata: HashMap<String, String>,
    categories: Vec<Id>,
}

impl Transaction {
    fn new(
        id: Id,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Id,
        destination: Id,
        budget: Option<Id>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categories: Vec<Id>,
    ) -> Self {
        Self {
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
        }
    }

    fn connection_with_account(&self, account: Id) -> bool {
        if account == self.source {
            return true;
        }
        if account == self.destination {
            return true;
        }
        false
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn amount(&self) -> Currency {
        self.amount.clone()
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(desc) => Some(desc),
            None => None,
        }
    }

    pub fn source(&self) -> &Id {
        &self.source
    }

    pub fn destination(&self) -> &Id {
        &self.destination
    }

    pub fn budget(&self) -> Option<&Id> {
        match &self.budget {
            Some(budget) => Some(budget),
            None => None,
        }
    }

    pub fn date(&self) -> &DateTime {
        &self.date
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn categories(&self) -> &Vec<Id> {
        &self.categories
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Recouring {
    /// start time and days
    Days(DateTime, usize),
    /// i.e. 3. of each month
    DayInMonth(u16),
    /// month and day
    Yearly(u8, u16),
}

impl std::fmt::Display for Recouring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yearly(month, day) => write!(f, "Yearly on {}.{}", day, month),
            Self::DayInMonth(day) => write!(f, "Day in month {}", day),
            Self::Days(date, days) => write!(f, "Every {} days starting from {}", days, date),
        }
    }
}

pub type Timespan = (Option<DateTime>, Option<DateTime>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Or<T, F> {
    One(T),
    Two(F),
}

pub trait FinanceManager: Send + Clone + Sized {
    fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + Send;

    fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + Send;

    fn get_accounts(&self) -> impl futures::Future<Output = Result<Vec<account::Account>>> + Send;

    fn get_account(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<account::Account>>> + Send;

    fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Result<Currency>> + Send;

    fn get_transaction(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Transaction>>> + Send;

    fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + Send;

    fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<Id>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categoris: Vec<Id>,
    ) -> impl futures::Future<Output = Result<Transaction>> + Send;

    fn update_transaction(
        &mut self,
        id: Id,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<Id>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categoris: Vec<Id>,
    ) -> impl futures::Future<Output = Result<Transaction>> + Send;

    fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + Send;

    fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + Send;

    fn update_budget(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Result<Budget>> + Send;

    fn get_budgets(&self) -> impl futures::Future<Output = Result<Vec<Budget>>> + Send;

    fn get_budget(&self, id: Id) -> impl futures::Future<Output = Result<Option<Budget>>> + Send;

    fn delete_transaction(&mut self, id: Id) -> impl futures::Future<Output = Result<()>> + Send;

    fn get_transactions(
        &self,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + Send;

    fn get_transactions_of_budget(
        &self,
        id: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + Send;

    fn get_current_budget_transactions(
        &self,
        budget: &Budget,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + Send {
        let (start, end) = match budget.timespan() {
            Recouring::Days(start, days) => {
                let since_start = chrono::Utc::now() - start;
                let a: i64 = since_start.num_days() / *days as i64;
                let timespan_start = *start + chrono::Duration::days(a * *days as i64);
                let timespan_end = timespan_start + chrono::Duration::days(*days as i64);
                (timespan_start, timespan_end)
            }
            Recouring::DayInMonth(day) => {
                let day_in_current_month = chrono::Utc::now().with_day(*day as u32).unwrap();
                if day_in_current_month > chrono::Utc::now() {
                    (
                        chrono::Utc::now().with_day(*day as u32 - 1).unwrap(),
                        chrono::Utc::now().with_day(*day as u32).unwrap(),
                    )
                } else {
                    (
                        chrono::Utc::now().with_day(*day as u32).unwrap(),
                        chrono::Utc::now().with_day(*day as u32 + 1).unwrap(),
                    )
                }
            }
            Recouring::Yearly(month, day) => {
                let current_year = chrono::Utc::now().year();
                let in_current_year = chrono::Utc
                    .with_ymd_and_hms(current_year, *month as u32, *day as u32, 0, 0, 0)
                    .unwrap();

                if in_current_year > chrono::Utc::now() {
                    (
                        chrono::Utc
                            .with_ymd_and_hms(current_year - 1, *month as u32, *day as u32, 0, 0, 0)
                            .unwrap(),
                        in_current_year,
                    )
                } else {
                    (
                        in_current_year,
                        chrono::Utc
                            .with_ymd_and_hms(current_year + 1, *month as u32, *day as u32, 0, 0, 0)
                            .unwrap(),
                    )
                }
            }
        };

        self.get_transactions_of_budget(*budget.id(), (Some(start), Some(end)))
    }

    fn get_current_budget_value(
        &self,
        budget: &Budget,
    ) -> impl futures::Future<Output = Result<Currency>> + Send {
        let transactions_future = self.get_current_budget_transactions(budget);
        async {
            let transactions = transactions_future.await?;
            let mut sum = Currency::Eur(0.0);
            for transaction in transactions {
                sum += transaction.amount();
            }
            Ok(sum)
        }
    }

    fn get_accounts_hash_map(
        &self,
    ) -> impl futures::Future<Output = Result<HashMap<Id, account::Account>>> + Send {
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

    fn get_categories(&self) -> impl futures::Future<Output = Result<Vec<Category>>> + Send;

    fn get_category(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<Category>>> + Send;

    fn create_category(
        &mut self,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + Send;

    fn update_category(
        &mut self,
        id: Id,
        name: String,
    ) -> impl futures::Future<Output = Result<Category>> + Send;
}
