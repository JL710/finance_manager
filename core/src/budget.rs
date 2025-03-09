use crate::{Currency, DateTime, Id, Timespan};
use anyhow::Result;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub total_value: Currency,
    pub timespan: Recurring,
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
}

impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
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
    let (start, end) = match &budget.timespan {
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
