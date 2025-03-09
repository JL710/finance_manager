use anyhow::Result;
use std::collections::HashMap;

mod finance_manager;
pub use finance_manager::FinanceManager;
#[cfg(feature = "test")]
pub mod finance_manager_test;

mod fm_controller;
pub use fm_controller::DeleteAccountError;
pub use fm_controller::FMController;

mod account_id;
pub use account_id::AccountId;
pub use iban_validate;

mod currency;
pub use currency::Currency;

pub mod account;

pub mod managers;
pub mod transaction_filter;

pub mod transaction;
pub use transaction::Transaction;

pub mod budget;
pub use budget::Budget;

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

pub type DateTime = time::OffsetDateTime;

#[cfg(not(target_arch = "wasm32"))]
pub fn get_local_timezone() -> Result<time::UtcOffset> {
    let utc_offset = tz::TimeZone::local()?
        .find_current_local_time_type()?
        .ut_offset();

    if utc_offset == 0 {
        #[cfg(unix)]
        return Ok(time::UtcOffset::UTC);
        #[cfg(not(unix))]
        return Ok(time::UtcOffset::current_local_offset()?);
    }

    Ok(time::UtcOffset::from_whole_seconds(utc_offset)?)
}

#[cfg(target_arch = "wasm32")]
pub fn get_local_timezone() -> Result<time::UtcOffset> {
    Ok(time::UtcOffset::from_whole_seconds(0)?)
}

pub type Id = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: Id,
    pub name: String,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Category {
    pub fn new(id: Id, name: String) -> Self {
        Self { id, name }
    }
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Eq for Category {}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
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

impl Sign {
    pub fn invert(&self) -> Self {
        match self {
            Sign::Positive => Sign::Negative,
            Sign::Negative => Sign::Positive,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Bill {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub value: Currency,
    pub transactions: HashMap<Id, Sign>,
    pub due_date: Option<DateTime>,
}

impl std::fmt::Display for Bill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Eq for Bill {}

impl std::hash::Hash for Bill {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.description.hash(state);
        self.value.hash(state);
        self.due_date.hash(state);
    }
}

impl Bill {
    pub fn new(
        id: Id,
        name: String,
        description: Option<String>,
        value: Currency,
        transactions: HashMap<Id, Sign>,
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
}

pub type Timespan = (Option<DateTime>, Option<DateTime>);

pub fn sum_up_transactions_by_day(
    mut transactions: Vec<Transaction>,
    sign_f: impl Fn(&Transaction) -> Sign,
) -> Vec<(DateTime, Currency)> {
    transactions.sort_by(|a, b| a.date.cmp(&b.date));

    let mut values: Vec<(DateTime, Currency)> = Vec::new();

    for transaction in transactions {
        let sign = (sign_f)(&transaction);
        let mut amount = match sign {
            Sign::Positive => transaction.amount,
            Sign::Negative => transaction.amount.negative(),
        };
        let date_with_offset = transaction.date.replace_time(time::Time::MIDNIGHT);
        // if it is not the first value only add it
        if !values.is_empty() {
            amount += values.last().unwrap().1.clone();
            let entry = values.last().unwrap().clone();
            // if it is the same day as the last entry, update the last entry
            if entry.0.to_offset(time::UtcOffset::UTC).date()
                == transaction.date.to_offset(time::UtcOffset::UTC).date()
            {
                let i = values.len() - 1;
                values[i] = (date_with_offset, amount);
                continue;
            }
        }
        // if it is the first value or a new day, add a new entry
        values.push((date_with_offset, amount));
    }

    values
}
