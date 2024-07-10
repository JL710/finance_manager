use anyhow::Result;
use chrono::{Datelike, TimeZone};
use std::collections::HashMap;

pub mod account;
#[cfg(feature = "ram")]
pub mod ram_finance_manager;
#[cfg(feature = "sqlite")]
pub mod sqlite_finange_manager;
pub mod transaction_filter;

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Id = u64;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
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

    pub fn from_currency_id(id: i32, amount: f64) -> Result<Self> {
        match id {
            1 => Ok(Currency::Eur(amount)),
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}

impl From<bool> for Sign {
    fn from(value: bool) -> Self {
        if value {
            Sign::Positive
        } else {
            Sign::Negative
        }
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
    budget: Option<(Id, Sign)>,
    date: DateTime,
    metadata: HashMap<String, String>,
    categories: Vec<(Id, Sign)>,
}

impl Transaction {
    fn new(
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

    pub fn budget(&self) -> Option<&(Id, Sign)> {
        self.budget.as_ref()
    }

    pub fn date(&self) -> &DateTime {
        &self.date
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn categories(&self) -> &Vec<(Id, Sign)> {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Bill {
    id: Id,
    name: String,
    description: Option<String>,
    value: Currency,
    transactions: Vec<(Id, Sign)>,
    due_date: Option<DateTime>,
}

impl Bill {
    pub fn new(
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            value,
            transactions,
            due_date,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    pub fn value(&self) -> &Currency {
        &self.value
    }

    pub fn transactions(&self) -> &Vec<(Id, Sign)> {
        &self.transactions
    }

    pub fn due_date(&self) -> &Option<DateTime> {
        &self.due_date
    }
}

pub type Timespan = (Option<DateTime>, Option<DateTime>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Or<T, F> {
    One(T),
    Two(F),
}

pub trait PrivateFinanceManager: Send + Clone + Sized {
    fn private_create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend;

    fn private_update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend;

    fn private_create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    fn private_update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend;

    /// Only get the sum of the transactions for the account at the given date.
    /// Do not include any offset or similar!
    fn private_get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend;
}

fn make_iban_bic_unified(content: Option<String>) -> Option<String> {
    content.map(|content| content.to_uppercase().replace(' ', ""))
}

pub trait FinanceManager: Send + Clone + Sized + PrivateFinanceManager {
    fn get_bills(&self) -> impl futures::Future<Output = Result<Vec<Bill>>> + MaybeSend;

    fn get_bill(&self, id: &Id) -> impl futures::Future<Output = Result<Option<Bill>>> + MaybeSend;

    fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> impl futures::Future<Output = Result<Bill>> + MaybeSend;

    fn delete_bill(&mut self, id: Id) -> impl futures::Future<Output = Result<()>> + MaybeSend;

    fn update_bill(
        &mut self,
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: Vec<(Id, Sign)>,
        due_date: Option<DateTime>,
    ) -> impl futures::Future<Output = Result<()>> + MaybeSend;

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

    fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend {
        self.private_create_asset_account(
            name,
            note,
            make_iban_bic_unified(iban),
            make_iban_bic_unified(bic),
            offset,
        )
    }

    fn update_asset_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
        offset: Currency,
    ) -> impl futures::Future<Output = Result<account::AssetAccount>> + MaybeSend {
        self.private_update_asset_account(
            id,
            name,
            note,
            make_iban_bic_unified(iban),
            make_iban_bic_unified(bic),
            offset,
        )
    }

    fn get_accounts(
        &self,
    ) -> impl futures::Future<Output = Result<Vec<account::Account>>> + MaybeSend;

    fn get_account(
        &self,
        id: Id,
    ) -> impl futures::Future<Output = Result<Option<account::Account>>> + MaybeSend;

    fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend {
        let future = self.private_get_account_sum(account, date);

        async move {
            let sum = future.await?;
            if let account::Account::AssetAccount(asset_account) = account {
                Ok(sum + asset_account.offset().clone())
            } else {
                Ok(sum)
            }
        }
    }

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
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
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
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<(Id, Sign)>,
        date: DateTime,
        metadata: HashMap<String, String>,
        categoris: Vec<(Id, Sign)>,
    ) -> impl futures::Future<Output = Result<Transaction>> + MaybeSend;

    fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend {
        self.private_create_book_checking_account(
            name,
            notes,
            make_iban_bic_unified(iban),
            make_iban_bic_unified(bic),
        )
    }

    fn update_book_checking_account(
        &mut self,
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = Result<account::BookCheckingAccount>> + MaybeSend {
        self.private_update_book_checking_account(
            id,
            name,
            note,
            make_iban_bic_unified(iban),
            make_iban_bic_unified(bic),
        )
    }

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

    fn get_current_budget_transactions(
        &self,
        budget: &Budget,
    ) -> impl futures::Future<Output = Result<Vec<Transaction>>> + MaybeSend {
        let (start, end) = match budget.timespan() {
            Recouring::Days(start, days) => {
                let since_start = chrono::Utc::now() - start;
                let a: i64 = since_start.num_days() / *days as i64;
                let timespan_start = *start + chrono::Duration::days(a * *days as i64);
                let timespan_end = timespan_start + chrono::Duration::days(*days as i64);
                (timespan_start, timespan_end)
            }
            Recouring::DayInMonth(day) => {
                let now = chrono::Utc::now();
                let day_in_current_month = now.with_day(*day as u32).unwrap();
                if day_in_current_month > now {
                    (
                        now.with_day(*day as u32)
                            .unwrap()
                            .with_month(now.month() - 1)
                            .unwrap(),
                        now.with_day(*day as u32).unwrap(),
                    )
                } else {
                    (
                        now.with_day(*day as u32).unwrap(),
                        now.with_day(*day as u32)
                            .unwrap()
                            .with_month(now.month() + 1)
                            .unwrap(),
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
    ) -> impl futures::Future<Output = Result<Currency>> + MaybeSend {
        let transactions_future = self.get_current_budget_transactions(budget);
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

pub fn sum_up_transactions_by_day(
    mut transactions: Vec<Transaction>,
    sign_f: impl Fn(&Transaction) -> Sign,
) -> Vec<(DateTime, Currency)> {
    transactions.sort_by(|a, b| a.date().cmp(b.date()));

    let mut values: Vec<(DateTime, Currency)> = Vec::new();

    for transaction in transactions {
        let time = chrono::Utc
            .with_ymd_and_hms(
                transaction.date().year(),
                transaction.date().month(),
                transaction.date().day(),
                0,
                0,
                0,
            )
            .unwrap();
        let sign = (sign_f)(&transaction);
        let mut amount = match sign {
            Sign::Positive => transaction.amount(),
            Sign::Negative => transaction.amount().negative(),
        };
        if !values.is_empty() {
            amount += values.last().unwrap().1.clone();
            let entry = values.last().unwrap().clone();
            if entry.0 == time {
                let i = values.len() - 1;
                values[i] = (time, amount);
            } else {
                values.push((time, amount));
            }
        } else {
            values.push((time, amount));
        }
    }

    values
}
