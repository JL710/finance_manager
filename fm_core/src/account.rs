use super::{Currency, Id};

#[derive(Debug, Clone)]
pub struct AssetAccount {
    id: Id,
    name: String,
    notes: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
}

impl AssetAccount {
    pub fn new(
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            notes: note,
            iban,
            bic,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn note(&self) -> Option<&str> {
        match &self.notes {
            Some(note) => Some(note),
            None => None,
        }
    }

    pub fn iban(&self) -> Option<&str> {
        match &self.iban {
            Some(content) => Some(content),
            None => None,
        }
    }

    pub fn bic(&self) -> Option<&str> {
        match &self.bic {
            Some(content) => Some(content),
            None => None,
        }
    }
}

impl From<AssetAccount> for Account {
    fn from(value: AssetAccount) -> Self {
        Account::AssetAccount(value)
    }
}

#[derive(Debug, Clone)]
pub struct BookCheckingAccount {
    id: Id,
    name: String,
    notes: Option<String>,
    iban: Option<String>,
    bic: Option<String>
}

#[derive(Debug, Clone)]
pub enum Account {
    AssetAccount(AssetAccount),
    BookCheckingAccount(BookCheckingAccount),
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Account::AssetAccount(acc) => match other {
                Account::AssetAccount(other_acc) => acc.id == other_acc.id,
                _ => false,
            },
            Account::BookCheckingAccount(acc) => match other {
                Account::BookCheckingAccount(other_acc) => acc.id == other_acc.id,
                _ => false,
            },
        }
    }
}

impl PartialEq<Id> for Account {
    fn eq(&self, other: &Id) -> bool {
        match self {
            Account::AssetAccount(acc) => acc.id == *other,
            Account::BookCheckingAccount(acc) => acc.id == *other,
        }
    }
}
