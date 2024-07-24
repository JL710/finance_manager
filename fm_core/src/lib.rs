use anyhow::Result;
use chrono::{Datelike, TimeZone, Timelike};
use std::collections::HashMap;

mod finance_manager;
pub use finance_manager::FinanceManager;

mod fm_controller;
pub use fm_controller::FMController;

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

    pub fn get_eur_num(&self) -> f64 {
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

impl PartialOrd for Currency {
    fn ge(&self, other: &Self) -> bool {
        self.get_eur_num() >= other.get_eur_num()
    }

    fn gt(&self, other: &Self) -> bool {
        self.get_eur_num() > other.get_eur_num()
    }

    fn le(&self, other: &Self) -> bool {
        self.get_eur_num() <= other.get_eur_num()
    }

    fn lt(&self, other: &Self) -> bool {
        self.get_eur_num() < other.get_eur_num()
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Currency {}

impl Ord for Currency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_eur_num().total_cmp(&other.get_eur_num())
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

pub fn calculate_budget_timespan(budget: &Budget, offset: i32, now: DateTime) -> Timespan {
    let now: chrono::prelude::DateTime<chrono::prelude::Utc> = now
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    let (start, end) = match budget.timespan() {
        Recouring::Days(start, days) => {
            let since_start = now - start;
            let a: i64 = since_start.num_days() / *days as i64;
            let timespan_start = *start + chrono::Duration::days(a * *days as i64);
            let timespan_end = timespan_start + chrono::Duration::days(*days as i64);
            (timespan_start, timespan_end)
        }
        Recouring::DayInMonth(day) => {
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
            let current_year = now.year();
            let in_current_year = chrono::Utc
                .with_ymd_and_hms(current_year, *month as u32, *day as u32, 0, 0, 0)
                .unwrap();

            if in_current_year > now {
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
    match offset.cmp(&0) {
        std::cmp::Ordering::Equal => (Some(start), Some(end - chrono::TimeDelta::seconds(1))),
        std::cmp::Ordering::Greater => {
            calculate_budget_timespan(budget, offset - 1, end + chrono::Duration::days(1))
        }
        std::cmp::Ordering::Less => {
            calculate_budget_timespan(budget, offset + 1, start - chrono::Duration::days(1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_budget_timespan() {
        let timespan = calculate_budget_timespan(
            &Budget::new(
                0,
                "Test".to_string(),
                None,
                Currency::Eur(0.0),
                Recouring::DayInMonth(1),
            ),
            0,
            chrono::Utc.with_ymd_and_hms(2020, 5, 3, 12, 10, 3).unwrap(),
        );
        assert_eq!(
            timespan,
            (
                Some(chrono::Utc.with_ymd_and_hms(2020, 5, 1, 0, 0, 0).unwrap()),
                Some(
                    chrono::Utc
                        .with_ymd_and_hms(2020, 5, 31, 23, 59, 59)
                        .unwrap()
                )
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
                Currency::Eur(0.0),
                Recouring::DayInMonth(1),
            ),
            2,
            chrono::Utc.with_ymd_and_hms(2020, 5, 3, 12, 10, 3).unwrap(),
        );
        assert_eq!(
            timespan,
            (
                Some(chrono::Utc.with_ymd_and_hms(2020, 7, 1, 0, 0, 0).unwrap()),
                Some(
                    chrono::Utc
                        .with_ymd_and_hms(2020, 7, 31, 23, 59, 59)
                        .unwrap()
                )
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
                Currency::Eur(0.0),
                Recouring::DayInMonth(1),
            ),
            -2,
            chrono::Utc.with_ymd_and_hms(2020, 5, 3, 12, 10, 3).unwrap(),
        );
        assert_eq!(
            timespan,
            (
                Some(chrono::Utc.with_ymd_and_hms(2020, 3, 1, 0, 0, 0).unwrap()),
                Some(
                    chrono::Utc
                        .with_ymd_and_hms(2020, 3, 31, 23, 59, 59)
                        .unwrap()
                )
            )
        );
    }
}
