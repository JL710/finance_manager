use super::action::Action;

use anyhow::Result;

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_in_terminal(
    importer: super::Importer<impl fm_core::FinanceManager, impl super::Parser>,
    finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
) -> Result<()> {
    let importer = Arc::new(Mutex::new(importer));
    loop {
        let next_action = importer.lock().await.next().await.unwrap();
        if let Some(action) = next_action {
            do_action(importer.clone(), action, finance_manager.clone()).await?;
        } else {
            break;
        }
    }
    Ok(())
}

async fn do_action(
    importer: Arc<Mutex<super::Importer<impl fm_core::FinanceManager, impl super::Parser>>>,
    action: Action,
    finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
) -> Result<()> {
    let mut actions = Vec::with_capacity(3);
    actions.push(action);
    while let Some(action) = actions.pop() {
        match action {
            Action::None => {}
            Action::TransactionCreated(transaction) => {
                let source = finance_manager
                    .lock()
                    .await
                    .get_account(*transaction.source())
                    .await
                    .unwrap()
                    .unwrap();
                let destination = finance_manager
                    .lock()
                    .await
                    .get_account(*transaction.destination())
                    .await
                    .unwrap()
                    .unwrap();
                println!(
                    "Title: {}\nDescription: {}\nValue: {}\nDate: {}\n\nSource: \n{}\n\nDestination: \n{}\n",
                    transaction.title(),
                    transaction.description().unwrap_or(""),
                    transaction.amount(),
                    transaction.date().to_offset(fm_core::get_local_timezone().unwrap()).format(&time::format_description::parse("[day].[month].[year]")?)?,
                    format_account(&source),
                    format_account(&destination)
                );
                break;
            }
            Action::TransactionExists(mut transaction_exists) => {
                decide_object_exists(
                    &mut transaction_exists,
                    |transaction, fm| async move {
                        let source = fm
                            .lock()
                            .await
                            .get_account(*transaction.source())
                            .await?
                            .unwrap();
                        let destination = fm
                            .lock()
                            .await
                            .get_account(*transaction.destination())
                            .await?
                            .unwrap();
                        Ok(format!(
                            "Title: {}\nDescription: {}\nValue: {}\nDate: {}\n\nSource: {}\n\nDestination: {}\n",
                            transaction.title(),
                            transaction.description().unwrap_or(""),
                            transaction.amount(),
                            transaction.date().to_offset(fm_core::get_local_timezone().unwrap()).format(&time::format_description::parse("[day].[month].[year]")?)?,
                            format_account(&source),
                            format_account(&destination)
                        ))
                    },
                    "The following transaction could already exists. What do you want to do?",
                    finance_manager.clone(),
                )
                .await?;
                actions.push(
                    importer
                        .lock()
                        .await
                        .perform(Action::TransactionExists(transaction_exists))
                        .await
                        .unwrap(),
                );
            }
            Action::DestinationAccountExists(mut destination_account_exists) => {
                decide_object_exists(
                    &mut destination_account_exists,
                    |acc, _| async move { Ok(format_account(&acc)) },
                    "The following Account could already exist. What do you want to do?",
                    finance_manager.clone(),
                )
                .await?;
                actions.push(
                    importer
                        .try_lock()
                        .unwrap()
                        .perform(Action::DestinationAccountExists(destination_account_exists))
                        .await
                        .unwrap(),
                );
            }
            Action::SourceAccountExists(mut source_account_exists) => {
                decide_object_exists(
                    &mut source_account_exists,
                    |acc, _| async move { Ok(format_account(&acc)) },
                    "The following Account could already exist. What do you want to do?",
                    finance_manager.clone(),
                )
                .await?;
                actions.push(
                    importer
                        .lock()
                        .await
                        .perform(Action::SourceAccountExists(source_account_exists))
                        .await
                        .unwrap(),
                );
            }
        }
    }
    Ok(())
}

async fn decide_object_exists<T: Clone, F, FM: fm_core::FinanceManager + 'static>(
    object_exists: &mut super::action::ObjectExists<T>,
    t_to_string: impl Fn(T, Arc<Mutex<fm_core::FMController<FM>>>) -> F,
    prompt: &str,
    finance_manager: Arc<Mutex<fm_core::FMController<FM>>>,
) -> Result<()>
where
    F: std::future::Future<Output = Result<String>>,
{
    println!("---------\n{}", prompt);
    println!(
        "You are making the decision for the following transaction:\n{}",
        format_transaction_entry(object_exists.transaction_entry())?
    );
    println!("You have do decide between the following options (enter the number or None):");
    for (i, item) in object_exists.possible_objects().iter().enumerate() {
        println!(
            "{}: \n{}",
            i,
            (t_to_string)(item.clone(), finance_manager.clone())
                .await
                .unwrap()
        );
    }
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input == "None" {
            object_exists.select_object(None);
            return Ok(());
        }
        let input = input.trim().parse::<isize>().expect("Expected a number");
        if input < object_exists.possible_objects().len() as isize && input >= 0 {
            object_exists.select_object(Some(
                object_exists.possible_objects()[input as usize].clone(),
            ));
            println!("Selected option {}.", input);
            return Ok(());
        } else {
            println!("Invalid input. Please try again.");
        }
    }
}

fn format_transaction_entry(entry: &super::TransactionEntry) -> Result<String> {
    Ok(format!(
        "Title: {}\nDescription: {}\nValue: {}\nSource IBAN: {}\nSource BIC: {}\nSource Name: {}\nDestination IBAN: {}\nDestination Name: {}\nDestination BIC: {}\nDate: {}\n",
        entry.title,
        entry.description,
        entry.value,
        entry.source_entry.iban(),
        entry.source_entry.bic().unwrap_or_default(),
        entry.source_entry.name().clone().unwrap_or_default(),
        entry.destination_entry.iban(),
        entry.destination_entry.bic().unwrap_or_default(),
        entry.destination_entry.name().clone().unwrap_or_default(),
        entry.date.to_offset(fm_core::get_local_timezone().unwrap()).format(&time::format_description::parse("[day].[month].[year]")?)?
    ))
}

fn format_account(account: &fm_core::account::Account) -> String {
    format!(
        "Name: {}\nDescription: {}\nIBAN: {}\nBIC: {}\n",
        account.name(),
        account.note().unwrap_or_default(),
        account
            .iban()
            .clone()
            .map_or(String::new(), |x| x.to_string()),
        account.bic().unwrap_or_default()
    )
}
