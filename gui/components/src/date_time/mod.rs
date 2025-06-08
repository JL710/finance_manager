pub mod date_input;
pub mod date_span_input;
pub mod date_time_input;
pub mod time_input;

use anyhow::{Context, Result};

type DateSpan = (Option<time::Date>, Option<time::Date>);

pub fn time_span_to_date_span(value: fm_core::Timespan) -> DateSpan {
    (value.0.map(|x| x.date()), value.1.map(|x| x.date()))
}

pub fn date_span_to_time_span(value: DateSpan, utc_offset: time::UtcOffset) -> fm_core::Timespan {
    (
        value.0.map(|date| {
            time::OffsetDateTime::new_in_offset(
                date,
                time::Time::from_hms(0, 0, 0).unwrap(),
                utc_offset,
            )
        }),
        value.1.map(|date| {
            time::OffsetDateTime::new_in_offset(
                date,
                time::Time::from_hms(23, 59, 59).unwrap(),
                utc_offset,
            )
        }),
    )
}
pub fn offset_to_primitive(value: time::OffsetDateTime) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(value.date(), value.time())
}

pub fn primitive_to_offset(
    value: time::PrimitiveDateTime,
    offset: time::UtcOffset,
) -> time::OffsetDateTime {
    time::OffsetDateTime::new_in_offset(value.date(), value.time(), offset)
}

/// returns year, month, day
fn parse_date_str_numbers(date_str: &str) -> Result<(u8, u8, i32)> {
    let splits = date_str
        .replace("/", ".")
        .split(".")
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if splits.len() != 3 {
        anyhow::bail!("Illegal date format");
    }

    Ok((
        splits[0]
            .parse()
            .context("Error while converting day to int")?,
        splits[1]
            .parse()
            .context("Error while converting month to int")?,
        splits[2]
            .parse()
            .context("Error while converting year to int")?,
    ))
}

// hour and minutes
fn parse_time_str(time_str: &str) -> Result<time::Time> {
    let splits = time_str.split(":").collect::<Vec<_>>();
    if splits.len() != 2 {
        anyhow::bail!("Error incorrect amount of time parameters");
    }
    time::Time::from_hms(
        splits[0]
            .parse()
            .context("Error while converting hour to int")?,
        splits[1]
            .parse()
            .context("Error while converting minute to int")?,
        0,
    )
    .context("Error while converting input to time")
}

pub fn parse_date_time_str(
    date_time_str: &str,
    utc_offset: time::UtcOffset,
) -> Result<time::OffsetDateTime> {
    let splits = date_time_str.split(" ").collect::<Vec<_>>();
    if splits.len() != 2 {
        anyhow::bail!("Error could not find date and time in text");
    }
    let date_numbers = parse_date_str_numbers(splits[0]).context("illegal date format")?;
    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::from_calendar_date(date_numbers.2, date_numbers.1.try_into()?, date_numbers.0)?,
        parse_time_str(splits[1]).context("illegal time format")?,
        utc_offset,
    ))
}

pub fn parse_date_str(date: &str) -> Result<time::Date> {
    let date_numbers = parse_date_str_numbers(date).context("illegal date format")?;

    Ok(time::Date::from_calendar_date(
        date_numbers.2,
        date_numbers.1.try_into()?,
        date_numbers.0,
    )?)
}

pub fn to_date_string(date: time::Date) -> String {
    date.format(&time::format_description::parse("[day].[month].[year]").unwrap())
        .unwrap()
}

pub fn to_date_time_string(date_time: time::PrimitiveDateTime) -> String {
    date_time
        .format(&time::format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap())
        .unwrap()
}

pub fn to_time_string(t: time::Time) -> String {
    t.format(&time::format_description::parse("[hour]:[minute]").unwrap())
        .unwrap()
}

pub fn add_months(date_time: time::OffsetDateTime, months: i32) -> time::OffsetDateTime {
    let mut months = date_time.date().month() as i32 + months;
    let mut year = date_time.year();

    while months > 12 {
        year += 1;
        months -= 12;
    }
    while months <= 0 {
        year -= 1;
        months += 12;
    }
    date_time
        .replace_year(year)
        .unwrap()
        .replace_month(time::Month::try_from(months as u8).unwrap())
        .unwrap()
}

#[derive(Debug, Copy, Clone)]
pub enum Shift {
    Duration(time::Duration),
    Month,
    Year,
}

fn apply_date_shift(date: time::Date, shift: Shift, positive: bool) -> time::Date {
    match shift {
        Shift::Duration(duration) => {
            if positive {
                date.checked_add(duration).unwrap()
            } else {
                date.checked_sub(duration).unwrap()
            }
        }
        Shift::Month => {
            let mut month = date.month() as u8;
            let mut year = date.year();
            if positive {
                if month == 12 {
                    month = 1;
                    year += 1;
                } else {
                    month += 1;
                }
            } else if month == 1 {
                month = 12;
                year -= 1;
            } else {
                month -= 1;
            }

            date.replace_year(year)
                .unwrap()
                .replace_day(date.day().min(time::util::days_in_month(
                    time::Month::try_from(month).unwrap(),
                    year,
                )))
                .unwrap()
                .replace_month(time::Month::try_from(month).unwrap())
                .unwrap()
        }
        Shift::Year => date
            .replace_year(date.year() + if positive { 1 } else { -1 })
            .unwrap(),
    }
}

fn apply_shift(
    date_time: time::OffsetDateTime,
    shift: Shift,
    positive: bool,
) -> time::OffsetDateTime {
    match shift {
        Shift::Duration(duration) => {
            if positive {
                date_time.checked_add(duration).unwrap()
            } else {
                date_time.checked_sub(duration).unwrap()
            }
        }
        Shift::Month => add_months(date_time, if positive { 1 } else { -1 }),
        Shift::Year => date_time
            .replace_year(date_time.year() + if positive { 1 } else { -1 })
            .unwrap(),
    }
}
