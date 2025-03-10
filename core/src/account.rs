use crate::Currency;

use super::{AccountId, Id};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Bic(String);

impl std::fmt::Display for Bic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Bic {
    pub fn new(bic: String) -> Self {
        Self(bic.to_uppercase().replace(' ', ""))
    }
}

impl From<String> for Bic {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a Bic> for &'a String {
    fn from(val: &'a Bic) -> Self {
        &val.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct AssetAccount {
    pub id: Id,
    pub name: String,
    pub note: Option<String>,
    pub iban: Option<AccountId>,
    pub bic: Option<Bic>,
    pub offset: super::Currency,
}

impl AssetAccount {
    pub fn new(
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
        offset: Currency,
    ) -> Self {
        Self {
            id,
            name,
            note,
            iban,
            bic,
            offset,
        }
    }
}

impl From<AssetAccount> for Account {
    fn from(value: AssetAccount) -> Self {
        Account::AssetAccount(value)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BookCheckingAccount {
    pub id: Id,
    pub name: String,
    pub note: Option<String>,
    pub iban: Option<AccountId>,
    pub bic: Option<Bic>,
}

impl std::fmt::Display for BookCheckingAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<BookCheckingAccount> for Account {
    fn from(val: BookCheckingAccount) -> Self {
        Account::BookCheckingAccount(val)
    }
}

impl BookCheckingAccount {
    pub fn new(
        id: Id,
        name: String,
        note: Option<String>,
        iban: Option<AccountId>,
        bic: Option<Bic>,
    ) -> Self {
        Self {
            id,
            name,
            note,
            iban,
            bic,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Account {
    AssetAccount(AssetAccount),
    BookCheckingAccount(BookCheckingAccount),
}

impl Account {
    pub fn name(&self) -> &str {
        match self {
            Account::AssetAccount(acc) => &acc.name,
            Account::BookCheckingAccount(acc) => &acc.name,
        }
    }

    pub fn note(&self) -> Option<&String> {
        match self {
            Account::AssetAccount(acc) => acc.note.as_ref(),
            Account::BookCheckingAccount(acc) => acc.note.as_ref(),
        }
    }

    pub fn id(&self) -> &Id {
        match self {
            Account::AssetAccount(acc) => &acc.id,
            Account::BookCheckingAccount(acc) => &acc.id,
        }
    }

    pub fn iban(&self) -> Option<&AccountId> {
        match self {
            Account::AssetAccount(acc) => acc.iban.as_ref(),
            Account::BookCheckingAccount(acc) => acc.iban.as_ref(),
        }
    }

    pub fn bic(&self) -> Option<&Bic> {
        match self {
            Account::AssetAccount(acc) => acc.bic.as_ref(),
            Account::BookCheckingAccount(acc) => acc.bic.as_ref(),
        }
    }
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Account::AssetAccount(acc) => write!(f, "{}", acc.name),
            Account::BookCheckingAccount(acc) => write!(f, "{}", acc.name),
        }
    }
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
