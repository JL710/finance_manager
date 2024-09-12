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
    categories: HashMap<Id, Sign>,
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
        categories: HashMap<Id, Sign>,
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

    pub fn categories(&self) -> &HashMap<Id, Sign> {
        &self.categories
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
        if !values.is_empty() {
            amount += values.last().unwrap().1.clone();
            let entry = values.last().unwrap().clone();
            if entry.0.to_offset(time::UtcOffset::UTC).date()
                == transaction.date().to_offset(time::UtcOffset::UTC).date()
            {
                let i = values.len() - 1;
                values[i] = (date_with_offset, amount);
            } else {
                values.push((date_with_offset, amount)); // FIXME: is this line really needed?
            }
        } else {
            values.push((date_with_offset, amount));
        }
    }

    values
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
                    now.replace_day(*day)?
                        .replace_month(now.month().previous())?, // FIXME: what if it is January?
                    now.replace_day(*day)?,
                )
            } else {
                (
                    now.replace_day(*day)?,
                    now.replace_day(*day)?.replace_month(now.month().next())?, // FIXME: what if it is December?
                )
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
