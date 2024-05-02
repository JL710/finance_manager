use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Import Source File
    #[clap(short, long)]
    source: String,
    /// Import Format
    #[clap(short, long, default_value = "CSV_CAMT_V2")]
    format: String,
    /// The url to bind to
    #[clap(short, long, default_value = "http://127.0.0.1:3000")]
    url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut finance_manager = fm_server::client::Client::new(args.url);
    match args.format.as_str() {
        "CSV_CAMT_V2" => parse_csv_camt_v2(&args.source, &mut finance_manager)
            .await
            .unwrap(),
        _ => eprintln!("Unknown format: {}", args.format),
    }
}

#[derive(Debug)]
struct TransactionEntry {
    account: String,
    date: fm_core::DateTime,
    value: f64,
    other_account: String,
    row_content: String,
    declared_use: String,
    bic: String,
    beneficiary_or_payer: String,
    booking_text: String,
}

impl TransactionEntry {
    fn from_strings(
        account: String,
        date: String,
        value: String,
        other_account: String,
        row_content: String,
        declared_use: String,
        bic: String,
        beneficiary_or_payer: String,
        booking_text: String,
    ) -> Result<Self> {
        Ok(Self {
            account,
            date: parse_to_datetime_short_year(&date)?,
            value: value.replace(",", ".").parse::<f64>().unwrap(),
            other_account,
            row_content,
            declared_use,
            bic,
            beneficiary_or_payer,
            booking_text,
        })
    }
}

async fn parse_csv_camt_v2(
    source: &str,
    finance_manager: &mut impl fm_core::FinanceManager,
) -> Result<()> {
    println!("Parsing CSV_CAMT_V2 file: {}", source);

    // read file from format ISO_8859_15
    let read_file = std::fs::read(source)?;
    let as_utf8 = encoding_rs::ISO_8859_15.decode(&read_file).0;

    // create csv reader
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(as_utf8.as_bytes());

    let transactions = finance_manager
        .get_transactions((None, None))
        .await
        .unwrap();
    let mut transaction_tuples = Vec::new();
    for transaction in transactions {
        let source_acc = finance_manager
            .get_account(*transaction.source())
            .await?
            .unwrap();
        let destination_acc = finance_manager
            .get_account(*transaction.destination())
            .await?
            .unwrap();
        transaction_tuples.push((transaction, source_acc, destination_acc));
    }
    let mut accounts = finance_manager.get_accounts().await?;

    let mut transaction_entries = Vec::new();

    // iterate over records
    for result in reader.records().into_iter() {
        let result = result?;

        if result.get(16).unwrap() != "Umsatz gebucht" {
            continue;
        }
        let account = result.get(0).unwrap();
        let date = result.get(1).unwrap();
        let value = result.get(14).unwrap();
        let other_account = result.get(12).unwrap();
        let declared_use = result.get(4).unwrap();
        let bic = result.get(13).unwrap();
        let beneficiary_or_payer = result.get(11).unwrap();
        let booking_text = result.get(3).unwrap();
        transaction_entries.push(TransactionEntry::from_strings(
            account.to_string(),
            date.to_string(),
            value.to_string(),
            other_account.to_string(),
            String::from_utf8(result.as_byte_record().as_slice().to_vec())?,
            declared_use.to_string(),
            bic.to_string(),
            beneficiary_or_payer.to_string(),
            booking_text.to_string(),
        )?);
    }

    'entry_loop: for entry in transaction_entries {
        for transaction in &transaction_tuples {
            if transaction.0.metadata().get("parser-row-content") == Some(&entry.row_content) {
                continue 'entry_loop;
            }
            if transaction.1.iban().is_none() || transaction.2.iban().is_none() {
                continue;
            }
            // check if amount is equal
            if transaction.0.amount().get_num() != entry.value {
                continue;
            }
            // check if date is equal
            if *transaction.0.date() != entry.date {
                continue;
            }
            // check if iban is equal
            if entry.value <= 0.0
                && transaction.1.iban().unwrap() != entry.account
                && transaction.2.iban().unwrap() != entry.other_account
            {
                continue;
            }
            if entry.value >= 0.0
                && transaction.1.iban().unwrap() != entry.other_account
                && transaction.2.iban().unwrap() != entry.account
            {
                continue;
            }

            // seem equal let the user decide
            // print transaction and ask user if it is the same
            println!(
                "\n\n\n\n\nTransaction already exists, skipping (y/n): {:?}\n\n{:?}",
                transaction.0, entry
            );
            if input() == "y" {
                continue 'entry_loop;
            }
        }

        let mut source = fm_core::Or::Two(if entry.value < 0.0 {
            entry.account.clone()
        } else {
            entry.other_account.clone()
        });
        let mut destination = fm_core::Or::Two(if entry.value < 0.0 {
            entry.other_account.clone()
        } else {
            entry.account.clone()
        });

        // find accounts
        for account in &accounts {
            if account.iban().is_none() {
                continue;
            }
            if entry.value >= 0.0 {
                // account is destination
                // other_account is source
                if account.iban().unwrap() == entry.account {
                    destination = fm_core::Or::One(*account.id());
                }
                if account.iban().unwrap() == entry.other_account {
                    source = fm_core::Or::One(*account.id());
                }
            } else {
                // other_account is destination
                // account is source
                if account.iban().unwrap() == entry.account {
                    source = fm_core::Or::One(*account.id());
                }
                if account.iban().unwrap() == entry.other_account {
                    destination = fm_core::Or::One(*account.id());
                }
            }
        }

        // create book_checking accounts
        for foo in [&mut source, &mut destination] {
            if let fm_core::Or::Two(acc) = foo {
                let account = finance_manager
                    .create_book_checking_account(
                        entry.beneficiary_or_payer.clone(),
                        None,
                        Some(entry.other_account.to_string()),
                        Some(entry.bic.clone()),
                    )
                    .await?;
                accounts.push(account.clone().into());
                *foo = fm_core::Or::One(account.id());
            }
        }

        // create new transaction
        let transaction = finance_manager
            .create_transaction(
                fm_core::Currency::Eur(entry.value.abs()),
                entry.booking_text,
                Some(format!(
                    "{}\n{}",
                    entry.declared_use, entry.beneficiary_or_payer
                )),
                source,
                destination,
                None,
                entry.date,
                HashMap::from([
                    (String::from("parser-row-content"), entry.row_content),
                    (
                        "parser-import-format".to_string(),
                        "CSV_CAMT_V2".to_string(),
                    ),
                ]),
                Vec::new(),
            )
            .await?;
    }

    Ok(())
}

fn input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn parse_to_datetime_short_year(date: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    use chrono::TimeZone;
    let date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%y")?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Ok(chrono::Utc.from_utc_datetime(&date))
}
