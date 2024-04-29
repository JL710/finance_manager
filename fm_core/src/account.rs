use super::Id;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            iban: iban.and_then(|x| Some(x.to_uppercase())),
            bic: bic.and_then(|x| Some(x.to_uppercase())),
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BookCheckingAccount {
    id: Id,
    name: String,
    notes: Option<String>,
    iban: Option<String>,
    bic: Option<String>,
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
        iban: Option<String>,
        bic: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            notes: note,
            iban: iban.and_then(|x| Some(x.to_uppercase())),
            bic: bic.and_then(|x| Some(x.to_uppercase())),
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Account {
    AssetAccount(AssetAccount),
    BookCheckingAccount(BookCheckingAccount),
}

impl Account {
    pub fn id(&self) -> Id {
        match self {
            Account::AssetAccount(acc) => acc.id,
            Account::BookCheckingAccount(acc) => acc.id,
        }
    }

    pub fn iban(&self) -> Option<&str> {
        match self {
            Account::AssetAccount(acc) => acc.iban(),
            Account::BookCheckingAccount(acc) => acc.iban(),
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
