use std::io::BufReader;

use anyhow::{Context, Result};
use csv_parser::CSVParser;
use fm_core::FinanceManager;

use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod csv_parser;
pub mod terminal_importer;

const VERSION: u8 = 1;
const METADATA_IMPORTER_VERSION: &str = "importer-version";
const METADATA_RAW_CONTENT: &str = "importer-raw-content";
const METADATA_IMPORT_FORMAT: &str = "importer-import-format";

#[derive(Debug, Clone, PartialEq)]
pub struct AccountEntry {
    name: Option<String>,
    iban: fm_core::AccountId,
    bic: Option<String>,
}

impl AccountEntry {
    pub fn new(name: Option<String>, iban: fm_core::AccountId, bic: Option<String>) -> Self {
        Self { name, iban, bic }
    }

    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn iban(&self) -> &fm_core::AccountId {
        &self.iban
    }

    pub fn bic(&self) -> Option<&str> {
        self.bic.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct TransactionEntry {
    raw_data: String,
    title: String,
    description: String,
    // Must be positive!
    value: fm_core::Currency,
    source_entry: AccountEntry,
    destination_entry: AccountEntry,
    date: fm_core::DateTime,
    source_account: Option<fm_core::account::Account>,
    destination_account: Option<fm_core::account::Account>,
}

impl TransactionEntry {
    pub fn new(
        raw_data: String,
        title: String,
        description: String,
        value: fm_core::Currency,
        source_entry: AccountEntry,
        destination_entry: AccountEntry,
        date: fm_core::DateTime,
    ) -> Result<Self> {
        if value.get_eur_num() < 0.0 {
            return Err(anyhow::anyhow!("Value must be positive"));
        }
        Ok(Self {
            raw_data,
            title,
            description,
            value,
            source_entry,
            destination_entry,
            date,
            source_account: None,
            destination_account: None,
        })
    }
}

pub mod action {
    use super::TransactionEntry;

    #[derive(Clone, Debug)]
    pub enum Action {
        None,
        TransactionCreated(fm_core::Transaction),
        TransactionExists(ObjectExists<fm_core::Transaction>),
        SourceAccountExists(ObjectExists<fm_core::account::Account>),
        DestinationAccountExists(ObjectExists<fm_core::account::Account>),
    }

    #[derive(Clone, Debug)]
    pub struct ObjectExists<T: Clone> {
        possible_objects: Vec<T>,
        pub(super) transaction_entry: TransactionEntry,
        selected_object_id: Option<fm_core::Id>,
        get_id: fn(&T) -> fm_core::Id,
    }

    impl<T: Clone> ObjectExists<T> {
        pub(super) fn new(
            possible_objects: Vec<T>,
            transaction_entry: TransactionEntry,
            get_id: fn(&T) -> fm_core::Id,
        ) -> Self {
            Self {
                possible_objects,
                transaction_entry,
                selected_object_id: None,
                get_id,
            }
        }

        pub fn possible_objects(&self) -> &Vec<T> {
            &self.possible_objects
        }

        pub fn select_object(&mut self, object: Option<T>) {
            let object_id = if let Some(object) = object {
                (self.get_id)(&object)
            } else {
                self.selected_object_id = None;
                return;
            };

            if !self
                .possible_objects
                .iter()
                .any(|t| (self.get_id)(t) == object_id)
            {
                panic!("Account with id {} is not a valid option", &object_id);
            }
            self.selected_object_id = Some(object_id);
        }

        pub fn selected_object(&self) -> Option<T> {
            if let Some(id) = self.selected_object_id {
                for object in &self.possible_objects {
                    if (self.get_id)(object) == id {
                        return Some(object.clone());
                    }
                }
                None
            } else {
                None
            }
        }

        pub fn transaction_entry(&self) -> &TransactionEntry {
            &self.transaction_entry
        }
    }
}

pub trait Parser {
    fn next_entry(&mut self)
        -> impl std::future::Future<Output = Result<Option<TransactionEntry>>>;

    fn format_name(&self) -> &str;
}

pub struct Importer<FM: fm_core::FinanceManager + 'static, P: Parser> {
    parser: P,
    fm_controller: Arc<Mutex<fm_core::FMController<FM>>>,
    cached_accounts: Vec<fm_core::account::Account>,
    cached_transactions: Vec<fm_core::Transaction>,
    saved_account_decisions: Vec<(AccountEntry, fm_core::account::Account)>,
}

impl<FM: fm_core::FinanceManager, P: Parser> Importer<FM, P> {
    pub async fn new(
        importer: P,
        fm_controller: Arc<Mutex<fm_core::FMController<FM>>>,
    ) -> Result<Self> {
        let cached_accounts = fm_controller.lock().await.get_accounts().await?;
        let cached_transactions = fm_controller
            .lock()
            .await
            .get_transactions_in_timespan((None, None))
            .await?;

        Ok(Self {
            parser: importer,
            fm_controller,
            cached_accounts,
            cached_transactions,
            saved_account_decisions: Vec::new(),
        })
    }

    pub async fn next(&mut self) -> Result<Option<action::Action>> {
        tracing::debug!("Next transaction entry");
        if let Some(mut transaction_entry) = self.parser.next_entry().await? {
            // check if the transactions exists
            if let Some(a) = self
                .transaction_exists(
                    &transaction_entry,
                    &self.cached_accounts,
                    self.parser.format_name(),
                )
                .await?
            {
                return Ok(Some(a));
            }
            // check source accounts
            if let Some(source_account_action) =
                self.process_source_account(&mut transaction_entry).await?
            {
                return Ok(Some(source_account_action));
            }
            // check other accounts
            if let Some(other_account_action) = self
                .process_destination_account(&mut transaction_entry)
                .await?
            {
                return Ok(Some(other_account_action));
            }
            // create transaction
            let _transaction = self
                .create_transaction(&transaction_entry)
                .await
                .context("Error while creating a transaction")?;

            Ok(Some(action::Action::None))
        } else {
            tracing::debug!("No transaction entries left");
            Ok(None)
        }
    }

    pub async fn perform(&mut self, processed_action: action::Action) -> Result<action::Action> {
        match processed_action {
            action::Action::None => return Ok(action::Action::None),
            action::Action::TransactionCreated(_) => {
                return Ok(action::Action::None);
            }
            action::Action::TransactionExists(object_exists) => {
                if object_exists.selected_object().is_some() {
                    return Ok(action::Action::None);
                }
            }
            action::Action::SourceAccountExists(object_exists) => {
                // create account if it does not exist
                let selected_account = if let Some(acc) = object_exists.selected_object() {
                    self.saved_account_decisions.push((
                        object_exists.transaction_entry.source_entry.clone(),
                        acc.clone(),
                    ));
                    acc
                } else {
                    self.create_book_checking_account(&object_exists.transaction_entry.source_entry)
                        .await?
                };

                // set account for transaction entry
                let mut transaction_entry = object_exists.transaction_entry;
                transaction_entry.source_account = Some(selected_account);

                // do the other phases
                // check other accounts
                if let Some(other_account_action) = self
                    .process_destination_account(&mut transaction_entry)
                    .await?
                {
                    return Ok(other_account_action);
                }

                // create transaction
                let _transaction = self
                    .create_transaction(&transaction_entry)
                    .await
                    .context("Error while creating a transaction")?;
            }
            action::Action::DestinationAccountExists(object_exists) => {
                // create account if it does not exist
                let selected_account = if let Some(acc) = object_exists.selected_object() {
                    self.saved_account_decisions.push((
                        object_exists.transaction_entry.destination_entry.clone(),
                        acc.clone(),
                    ));
                    acc
                } else {
                    self.create_book_checking_account(
                        &object_exists.transaction_entry.destination_entry,
                    )
                    .await?
                };

                // set account for transaction entry
                let mut transaction_entry = object_exists.transaction_entry;
                transaction_entry.destination_account = Some(selected_account);

                // create transaction
                let _transaction = self
                    .create_transaction(&transaction_entry)
                    .await
                    .context("Error while creating a transaction")?;
            }
        }
        Ok(action::Action::None)
    }

    async fn process_source_account(
        &mut self,
        transaction_entry: &mut TransactionEntry,
    ) -> Result<Option<action::Action>> {
        self.process_account(
            transaction_entry,
            |t| &t.source_entry,
            |t, a| t.source_account = a,
            action::Action::SourceAccountExists,
        )
        .await
    }

    async fn process_destination_account(
        &mut self,
        transaction_entry: &mut TransactionEntry,
    ) -> Result<Option<action::Action>> {
        self.process_account(
            transaction_entry,
            |t| &t.destination_entry,
            |t, a| t.destination_account = a,
            action::Action::DestinationAccountExists,
        )
        .await
    }

    /// Check if source account exists and set it for the transaction_entry.
    /// If not sure if the account exists, return a [`action::Action`] for the decision.
    /// If it does not exist, create it.
    async fn process_account(
        &mut self,
        transaction_entry: &mut TransactionEntry,
        account_entry: impl Fn(&TransactionEntry) -> &AccountEntry,
        set_account: impl Fn(&mut TransactionEntry, Option<fm_core::account::Account>),
        exists_action: impl Fn(action::ObjectExists<fm_core::account::Account>) -> action::Action,
    ) -> Result<Option<action::Action>> {
        match self
            .account_exists((account_entry)(transaction_entry))
            .await
        {
            AccountExistsResult::NotFond => {
                // create account
                let account = self
                    .create_book_checking_account((account_entry)(transaction_entry))
                    .await?;
                tracing::info!("Account created: {:?}", account);
                (set_account)(transaction_entry, Some(account.clone()));
            }
            AccountExistsResult::Found(account) => {
                (set_account)(transaction_entry, Some(account));
            }
            AccountExistsResult::PossibleAccounts(accounts) => {
                // create action to select account
                return Ok(Some((exists_action)(action::ObjectExists::new(
                    accounts,
                    transaction_entry.clone(),
                    |a| *a.id(),
                ))));
            }
        }
        Ok(None)
    }

    async fn account_exists(&self, account_entry: &AccountEntry) -> AccountExistsResult {
        // check if account is in saved account decisions
        for (entry, account) in &self.saved_account_decisions {
            if entry == account_entry {
                tracing::debug!("used account from saved account decisions {:?}", account);
                return AccountExistsResult::Found(account.clone());
            }
        }

        // check for accounts in account cache
        let mut possible_accounts = Vec::new();

        for account in &self.cached_accounts {
            if let Some(account_iban) = account.iban() {
                if account_entry.iban() == account_iban
                    && if let Some(entry_name) = account_entry.name() {
                        entry_name == account.name()
                    } else {
                        true
                    }
                {
                    return AccountExistsResult::Found(account.clone());
                } else if account_entry.iban() == account_iban {
                    possible_accounts.push(account.clone());
                }
            }
        }

        if possible_accounts.is_empty() {
            AccountExistsResult::NotFond
        } else {
            AccountExistsResult::PossibleAccounts(possible_accounts)
        }
    }

    async fn create_book_checking_account(
        &mut self,
        account_entry: &AccountEntry,
    ) -> Result<fm_core::account::Account> {
        let account = self
            .fm_controller
            .lock()
            .await
            .create_book_checking_account(
                account_entry
                    .name()
                    .clone()
                    .unwrap_or(account_entry.iban().to_string()),
                None,
                Some(account_entry.iban().to_owned()),
                account_entry.bic().map(|s| s.to_owned()),
            )
            .await?;

        self.cached_accounts.push(account.clone().into());

        tracing::info!("Account created: {:?}", account);

        Ok(account.into())
    }

    /// Check if the transaction already exists in the finance manager.
    /// If the transaction exists, return an action to be performed.
    /// If the transaction does not exist, return None.
    async fn transaction_exists(
        &self,
        transaction_entry: &TransactionEntry,
        accounts: &[fm_core::account::Account],
        format_name: &str,
    ) -> Result<Option<action::Action>> {
        let mut possible_transactions = Vec::new();

        for transaction in &self.cached_transactions {
            // check for importer specific fields
            if let Some(parser_content) = transaction.metadata().get(METADATA_RAW_CONTENT) {
                if let Some(import_format) = transaction.metadata().get(METADATA_IMPORT_FORMAT) {
                    if *parser_content == transaction_entry.raw_data
                        && *import_format == format_name
                    {
                        return Ok(Some(action::Action::None));
                    }
                }
            }

            let source_acc = match accounts
                .iter()
                .find(|a| a.id() == transaction.id())
                .cloned()
            {
                Some(acc) => acc,
                None => continue,
            };

            let destination_acc = match accounts.iter().find(|a| a.id() == transaction.id()) {
                Some(acc) => acc,
                None => continue,
            };

            // check for general fields
            if transaction.amount() == transaction_entry.value
            && transaction.date().replace_offset(time::UtcOffset::UTC).date() == transaction_entry.date.replace_offset(time::UtcOffset::UTC).date()
            // check if source iban is equal
            && if let Some(source_iban) = source_acc.iban() {
                source_iban == transaction_entry.source_entry.iban()
            } else {
                false
            }
            // check if destination iban is equal
            && if let Some(destination_iban) = destination_acc.iban() {
                destination_iban == transaction_entry.destination_entry.iban()
            } else {
                false
            }
            // check if source bic is equal
            && if let Some(source_bic) = source_acc.bic() {
                if let Some(entry_source_bic) = transaction_entry.source_entry.bic() {
                    source_bic == entry_source_bic
                } else {
                    true
                }
            } else {
                true
            }
            // check if destination bic is equal
            && if let Some(destination_bic) = destination_acc.bic() {
                if let Some(entry_destination_bic) = transaction_entry.destination_entry.bic() {
                    destination_bic == entry_destination_bic
                } else {
                    true
                }
            } else {
                true
            } {
                possible_transactions.push(transaction.clone());
            }
        }

        if !possible_transactions.is_empty() {
            return Ok(Some(action::Action::TransactionExists(
                action::ObjectExists::new(possible_transactions, transaction_entry.clone(), |t| {
                    *t.id()
                }),
            )));
        }

        Ok(None)
    }

    async fn create_transaction(
        &mut self,
        transaction_entry: &TransactionEntry,
    ) -> Result<fm_core::Transaction> {
        // figure out who is the source and who is the destination
        let source = transaction_entry
            .source_account
            .as_ref()
            .map(|a| *a.id())
            .unwrap();
        let destination = transaction_entry
            .destination_account
            .as_ref()
            .map(|a| *a.id())
            .unwrap();
        let transaction = self
            .fm_controller
            .lock()
            .await
            .create_transaction(
                transaction_entry.value.clone(),
                transaction_entry.title.clone(),
                Some(transaction_entry.description.clone()),
                source,
                destination,
                None,
                transaction_entry.date,
                HashMap::from([
                    (
                        METADATA_RAW_CONTENT.to_string(),
                        transaction_entry.raw_data.clone(),
                    ),
                    (
                        METADATA_IMPORT_FORMAT.to_string(),
                        "CSV_CAMT_V2".to_string(),
                    ),
                    (METADATA_IMPORTER_VERSION.to_string(), VERSION.to_string()),
                ]),
                HashMap::new(),
            )?
            .await?;
        self.cached_transactions.push(transaction.clone());

        tracing::info!("Transaction created: {:?}", transaction);

        Ok(transaction)
    }
}

enum AccountExistsResult {
    NotFond,
    Found(fm_core::account::Account),
    PossibleAccounts(Vec<fm_core::account::Account>),
}

pub async fn csv_camt_v2_importer<FM: FinanceManager>(
    data: BufReader<&[u8]>,
    fm_controller: Arc<Mutex<fm_core::FMController<FM>>>,
) -> Result<Importer<FM, CSVParser>> {
    Importer::new(csv_parser::csv_camt_v2_parser(data)?, fm_controller).await
}
