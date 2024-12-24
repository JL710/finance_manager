use super::{Currency, DateTime, Id, Sign};
use std::collections::HashMap;

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
    categories: HashMap<Id, Sign>,
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

    pub fn categories(&self) -> &HashMap<Id, Sign> {
        &self.categories
    }
}
