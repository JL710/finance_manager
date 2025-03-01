pub mod date_input;
pub mod date_time_input;
pub mod time_input;
pub mod timespan_input;

use anyhow::{Context, Result};

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
    return time::Time::from_hms(
        splits[0]
            .parse()
            .context("Error while converting hour to int")?,
        splits[1]
            .parse()
            .context("Error while converting minute to int")?,
        0,
    )
    .context("Error while converting input to time");
}

pub fn parse_date_time_str(date_time_str: &str) -> Result<time::OffsetDateTime> {
    let splits = date_time_str.split(" ").collect::<Vec<_>>();
    if splits.len() != 2 {
        anyhow::bail!("Error could not find date and time in text");
    }
    let date_numbers = parse_date_str_numbers(splits[0]).context("illegal date format")?;
    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::from_calendar_date(date_numbers.2, date_numbers.1.try_into()?, date_numbers.0)?,
        parse_time_str(splits[1]).context("illegal time format")?,
        fm_core::get_local_timezone()?,
    ))
}

pub fn parse_date_str(date: &str, h: u8, m: u8, s: u8) -> Result<time::OffsetDateTime> {
    let date_numbers = parse_date_str_numbers(date).context("illegal date format")?;

    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::from_calendar_date(date_numbers.2, date_numbers.1.try_into()?, date_numbers.0)?,
        time::Time::from_hms(h, m, s)?,
        fm_core::get_local_timezone()?,
    ))
}

pub fn to_date_string(date_time: fm_core::DateTime) -> String {
    date_time
        .to_offset(fm_core::get_local_timezone().unwrap())
        .format(&time::format_description::parse("[day].[month].[year]").unwrap())
        .unwrap()
}

pub fn to_date_time_string(date_time: fm_core::DateTime) -> String {
    date_time
        .to_offset(fm_core::get_local_timezone().unwrap())
        .format(&time::format_description::parse("[day].[month].[year] [hour]:[minute]").unwrap())
        .unwrap()
}

pub fn to_time_string(t: time::Time) -> String {
    t.format(&time::format_description::parse("[hour]:[minute]").unwrap())
        .unwrap()
}
