pub mod date_input;
pub mod timespan_input;

pub fn parse_date_to_datetime(
    date: &str,
    h: u8,
    m: u8,
    s: u8,
) -> anyhow::Result<time::OffsetDateTime> {
    let mut splits = date
        .replace("/", ".")
        .split(".")
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if splits.len() != 3 {
        anyhow::bail!("Illegal date format");
    }
    if splits[0].len() == 1 {
        splits[0] = format!("0{}", splits[0]);
    }
    if splits[1].len() == 1 {
        splits[1] = format!("0{}", splits[1]);
    }

    Ok(time::OffsetDateTime::new_in_offset(
        time::Date::parse(
            format!("{}.{}.{}", splits[0], splits[1], splits[2]).as_str(),
            &time::format_description::parse("[day].[month].[year]")?,
        )?,
        time::Time::from_hms(h, m, s).unwrap(),
        fm_core::get_local_timezone()?,
    ))
}

pub fn date_time_to_date_string(date_time: fm_core::DateTime) -> String {
    date_time
        .to_offset(fm_core::get_local_timezone().unwrap())
        .format(&time::format_description::parse("[day].[month].[year]").unwrap())
        .unwrap()
}
