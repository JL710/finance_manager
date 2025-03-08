use super::{Currency, DateTime, Id, Sign};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub id: Id,
    pub amount: Currency, // if amount is positive the money is added to the account. If negative it is removed
    pub title: String,
    pub description: Option<String>,
    pub source: Id,
    pub destination: Id,
    pub budget: Option<(Id, Sign)>,
    pub date: DateTime,
    pub metadata: HashMap<String, String>,
    pub categories: HashMap<Id, Sign>,
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Transaction {
    #[allow(clippy::too_many_arguments)]
    #[allow(unused)]
    pub(super) fn new(
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

    pub(super) fn connection_with_account(&self, account: Id) -> bool {
        if account == self.source {
            return true;
        }
        if account == self.destination {
            return true;
        }
        false
    }
}
