use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};

enum IterOption {
    Ignored,
    Entry(Box<crate::TransactionEntry>),
}

#[allow(clippy::type_complexity)]
pub struct CSVParser<'a> {
    data: BufReader<&'a [u8]>,
    format_name: String,
    ignore_entry: Box<dyn Fn(&csv::StringRecord) -> bool>,
    title: Box<dyn Fn(&csv::StringRecord) -> String>,
    value: Box<dyn Fn(&csv::StringRecord) -> fm_core::Currency>,
    description: Box<dyn Fn(&csv::StringRecord) -> String>,
    source_iban: Box<dyn Fn(&csv::StringRecord) -> String>,
    source_name: Box<dyn Fn(&csv::StringRecord) -> Option<String>>,
    source_bic: Box<dyn Fn(&csv::StringRecord) -> Option<String>>,
    other_iban: Box<dyn Fn(&csv::StringRecord) -> String>,
    other_name: Box<dyn Fn(&csv::StringRecord) -> Option<String>>,
    other_bic: Box<dyn Fn(&csv::StringRecord) -> Option<String>>,
    date: Box<dyn Fn(&csv::StringRecord) -> fm_core::DateTime>,
    delimiter: u8,
}

impl<'a> CSVParser<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: BufReader<&'a [u8]>,
        format_name: String,
        delimiter: u8,
        ignore_entry: impl Fn(&csv::StringRecord) -> bool + 'static,
        title: impl Fn(&csv::StringRecord) -> String + 'static,
        value: impl Fn(&csv::StringRecord) -> fm_core::Currency + 'static,
        description: impl Fn(&csv::StringRecord) -> String + 'static,
        source_iban: impl Fn(&csv::StringRecord) -> String + 'static,
        source_name: impl Fn(&csv::StringRecord) -> Option<String> + 'static,
        source_bic: impl Fn(&csv::StringRecord) -> Option<String> + 'static,
        other_iban: impl Fn(&csv::StringRecord) -> String + 'static,
        other_name: impl Fn(&csv::StringRecord) -> Option<String> + 'static,
        other_bic: impl Fn(&csv::StringRecord) -> Option<String> + 'static,
        date: impl Fn(&csv::StringRecord) -> fm_core::DateTime + 'static,
    ) -> Result<Self> {
        // skip the first line because it is the header
        let mut data = data;
        data.read_line(&mut String::new())?;

        Ok(Self {
            data,
            delimiter,
            ignore_entry: Box::new(ignore_entry),
            title: Box::new(title),
            value: Box::new(value),
            description: Box::new(description),
            source_iban: Box::new(source_iban),
            source_name: Box::new(source_name),
            source_bic: Box::new(source_bic),
            other_iban: Box::new(other_iban),
            other_name: Box::new(other_name),
            other_bic: Box::new(other_bic),
            date: Box::new(date),
            format_name,
        })
    }

    /// If Option is None, the iteration is over.
    async fn next(&mut self) -> Result<Option<IterOption>> {
        let mut raw = String::new();
        self.data.read_line(&mut raw)?;
        // remove trailing newlines -> try to remove all kind of new lines -> see more https://en.wikipedia.org/wiki/Newline#Representation
        raw = raw
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .trim_end_matches('\n')
            .to_string();

        let record = if let Some(r) = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(self.delimiter)
            .from_reader(raw.as_bytes())
            .records()
            .next()
        {
            match r {
                Ok(r) => r,
                Err(e) => return Err(e.into()),
            }
        } else {
            return Ok(None);
        };

        // return early if the entry should be ignored
        if (self.ignore_entry)(&record) {
            return Ok(Some(IterOption::Ignored));
        }

        let value = (self.value)(&record);

        let (source_iban, source_name, source_bic) = if value.get_eur_num() < 0.0 {
            (
                (self.source_iban)(&record),
                (self.source_name)(&record),
                (self.source_bic)(&record),
            )
        } else {
            (
                (self.other_iban)(&record),
                (self.other_name)(&record),
                (self.other_bic)(&record),
            )
        };

        let (destination_iban, destination_name, destination_bic) = if value.get_eur_num() < 0.0 {
            (
                (self.other_iban)(&record),
                (self.other_name)(&record),
                (self.other_bic)(&record),
            )
        } else {
            (
                (self.source_iban)(&record),
                (self.source_name)(&record),
                (self.source_bic)(&record),
            )
        };

        Ok(Some(IterOption::Entry(Box::new(
            super::TransactionEntry::new(
                raw,
                (self.title)(&record),
                (self.description)(&record),
                if value.get_eur_num() < 0.0 {
                    value.negative()
                } else {
                    value.clone()
                },
                super::AccountEntry::new(source_name, source_iban.parse()?, source_bic),
                super::AccountEntry::new(
                    destination_name,
                    destination_iban.parse()?,
                    destination_bic,
                ),
                (self.date)(&record),
            )?,
        ))))
    }
}

impl<'a> super::Parser for CSVParser<'a> {
    async fn next_entry(&mut self) -> Result<Option<crate::TransactionEntry>> {
        loop {
            match self.next().await? {
                Some(IterOption::Entry(entry)) => return Ok(Some(*entry)),
                Some(IterOption::Ignored) => continue,
                None => return Ok(None),
            }
        }
    }

    fn format_name(&self) -> &str {
        &self.format_name
    }
}

pub fn csv_camt_v2_parser(data: BufReader<&[u8]>) -> Result<CSVParser> {
    pub fn parse_to_datetime(date: &str) -> anyhow::Result<fm_core::DateTime> {
        let mut parsed = time::parsing::Parsed::new();
        parsed.parse_items(
            date.as_bytes(),
            &time::format_description::parse("[day].[month].[year repr:last_two]")
                .context("Could not create format description")?,
        )?;

        Ok(time::OffsetDateTime::new_in_offset(
            time::Date::from_calendar_date(
                parsed.year_last_two().unwrap() as i32 + 2000, // add 2000 for the year because the year is only the last two digits
                parsed.month().unwrap(),
                parsed.day().unwrap().into(),
            )
            .context("Could not parse date")?,
            time::Time::MIDNIGHT,
            fm_core::get_local_timezone()?,
        ))
    }

    CSVParser::new(
        data,
        "CSV_CAMT_V2".to_string(),
        b';',
        |record| record.get(16).unwrap() != "Umsatz gebucht",
        |record| record.get(3).unwrap().to_string(),
        |record| {
            if record.get(15).unwrap() != "EUR" {
                panic!("Currency is not EUR");
            }
            record
                .get(14)
                .unwrap()
                .replace(',', ".")
                .parse::<f64>()
                .unwrap()
                .into()
        },
        |record| format!("{}\n{}", record.get(4).unwrap(), record.get(11).unwrap()),
        |record| record.get(0).unwrap().to_string(),
        |_| None,
        |_| None,
        |record| record.get(12).unwrap().to_string(),
        |record| Some(record.get(11).unwrap().to_string()),
        |record| Some(record.get(13).unwrap().to_string()),
        |record| parse_to_datetime(record.get(1).unwrap()).unwrap(),
    )
}
