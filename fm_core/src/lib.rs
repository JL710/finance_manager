use anyhow::Result;
use std::collections::HashMap;

mod finance_manager;
pub use finance_manager::FinanceManager;

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

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

pub type DateTime = time::OffsetDateTime;

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

pub type Id = u64;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    id: Id,
    name: String,
    description: Option<String>,
    total_value: Currency,
    timespan: Recurring,
}

impl Budget {
    pub fn new(
        id: Id,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recurring,
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

    pub fn timespan(&self) -> &Recurring {
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

impl Sign {
    pub fn invert(&self) -> Self {
        match self {
            Sign::Positive => Sign::Negative,
            Sign::Negative => Sign::Positive,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Recurring {
    /// start time and days
    Days(DateTime, usize),
    /// i.e. 3. of each month
    DayInMonth(u8),
    /// month and day
    Yearly(u8, u8),
}

impl std::fmt::Display for Recurring {
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
    transactions: HashMap<Id, Sign>,
    due_date: Option<DateTime>,
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

    pub fn transactions(&self) -> &HashMap<Id, Sign> {
        &self.transactions
    }

    pub fn due_date(&self) -> &Option<DateTime> {
        &self.due_date
    }
}

pub type Timespan = (Option<DateTime>, Option<DateTime>);

pub fn sum_up_transactions_by_day(
    mut transactions: Vec<Transaction>,
    sign_f: impl Fn(&Transaction) -> Sign,
) -> Vec<(DateTime, Currency)> {
    transactions.sort_by(|a, b| a.date().cmp(b.date()));

    let mut values: Vec<(DateTime, Currency)> = Vec::new();

    for transaction in transactions {
        let sign = (sign_f)(&transaction);
        let mut amount = match sign {
            Sign::Positive => transaction.amount(),
            Sign::Negative => transaction.amount().negative(),
        };
        let date_with_offset = (*transaction.date()).replace_time(time::Time::MIDNIGHT);
        // if it is not the first value only add it
        if !values.is_empty() {
            amount += values.last().unwrap().1.clone();
            let entry = values.last().unwrap().clone();
            // if it is the same day as the last entry, update the last entry
            if entry.0.to_offset(time::UtcOffset::UTC).date()
                == transaction.date().to_offset(time::UtcOffset::UTC).date()
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

fn next_month(datetime: DateTime) -> DateTime {
    let month = datetime.month().next();
    let year = if month == time::Month::January {
        datetime.year() + 1
    } else {
        datetime.year()
    };
    datetime
        .replace_month(month)
        .unwrap()
        .replace_year(year)
        .unwrap()
}

fn previus_month(datetime: DateTime) -> DateTime {
    let month = datetime.month().previous();
    let year = if month == time::Month::December {
        datetime.year() - 1
    } else {
        datetime.year()
    };
    datetime
        .replace_month(month)
        .unwrap()
        .replace_year(year)
        .unwrap()
}

pub fn calculate_budget_timespan(budget: &Budget, offset: i32, now: DateTime) -> Result<Timespan> {
    let now = now.replace_time(time::Time::MIDNIGHT);
    let (start, end) = match budget.timespan() {
        Recurring::Days(start, days) => {
            let since_start = now - *start;
            let a: i64 = since_start.whole_days() / *days as i64; // FIXME: what is this "a" for again?
            let timespan_start = *start + time::Duration::days(a * *days as i64);
            let timespan_end = timespan_start + time::Duration::days(*days as i64);
            (timespan_start, timespan_end)
        }
        Recurring::DayInMonth(day) => {
            let day_in_current_month = now.replace_day(*day)?;
            if day_in_current_month > now {
                (
                    previus_month(now.replace_day(*day)?),
                    now.replace_day(*day)?,
                )
            } else {
                (now.replace_day(*day)?, next_month(now.replace_day(*day)?))
            }
        }
        Recurring::Yearly(month, day) => {
            let current_year = now.year();
            let in_current_year = time::OffsetDateTime::new_utc(
                now.date()
                    .replace_day(*day)?
                    .replace_month((*month).try_into()?)?,
                time::Time::MIDNIGHT,
            );

            if in_current_year > now {
                (
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(
                            current_year - 1,
                            (*month).try_into()?,
                            *day,
                        )?,
                        time::Time::MIDNIGHT,
                    ),
                    in_current_year,
                )
            } else {
                (
                    in_current_year,
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(
                            current_year + 1,
                            (*month).try_into()?,
                            *day,
                        )?,
                        time::Time::MIDNIGHT,
                    ),
                )
            }
        }
    };
    match offset.cmp(&0) {
        std::cmp::Ordering::Equal => Ok((Some(start), Some(end - time::Duration::seconds(1)))),
        std::cmp::Ordering::Greater => {
            calculate_budget_timespan(budget, offset - 1, end + time::Duration::days(1))
        }
        std::cmp::Ordering::Less => {
            calculate_budget_timespan(budget, offset + 1, start - time::Duration::days(1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::*;

    #[test]
    fn test_calculate_budget_timespan() {
        let timespan = calculate_budget_timespan(
            &Budget::new(
                0,
                "Test".to_string(),
                None,
                Currency::default(),
                Recurring::DayInMonth(1),
            ),
            0,
            datetime!(2020-5-3 12:10:03 UTC),
        )
        .unwrap();
        assert_eq!(
            timespan,
            (
                Some(datetime!(2020-5-1 0:0:0 UTC)),
                Some(datetime!(2020-5-31 23:59:59 UTC))
            )
        );
    }

    #[test]
    fn test_calculate_budget_timespan_positive_offset() {
        let timespan = calculate_budget_timespan(
            &Budget::new(
                0,
                "Test".to_string(),
                None,
                Currency::default(),
                Recurring::DayInMonth(1),
            ),
            2,
            datetime!(2020-5-3 12:10:03 UTC),
        )
        .unwrap();
        assert_eq!(
            timespan,
            (
                Some(datetime!(2020-7-1 0:0:0 UTC)),
                Some(datetime!(2020-7-31 23:59:59 UTC))
            )
        );
    }

    #[test]
    fn test_calculate_budget_timespan_negative_offset() {
        let timespan = calculate_budget_timespan(
            &Budget::new(
                0,
                "Test".to_string(),
                None,
                Currency::default(),
                Recurring::DayInMonth(1),
            ),
            -2,
            datetime!(2020-5-3 12:10:3 UTC),
        )
        .unwrap();
        assert_eq!(
            timespan,
            (
                Some(datetime!(2020-03-01 0:0:0 UTC)),
                Some(datetime!(2020-03-31 23:59:59 UTC))
            )
        );
    }
}
