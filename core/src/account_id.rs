use iban_validate::IbanLike;
use std::str::FromStr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum AccountId {
    Iban(iban_validate::Iban),
    /// For cases where no valid iban but something else is available
    Other(String),
}

impl AccountId {
    pub fn electronic_str(&self) -> &str {
        match self {
            AccountId::Iban(iban) => iban.electronic_str(),
            AccountId::Other(other) => other,
        }
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountId::Iban(iban) => write!(f, "{}", iban),
            AccountId::Other(other) => write!(f, "{}", other),
        }
    }
}

impl FromStr for AccountId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(iban) = s.trim().to_uppercase().replace(" ", "").parse() {
            Ok(AccountId::Iban(iban))
        } else {
            Ok(AccountId::Other(s.to_string()))
        }
    }
}

impl From<String> for AccountId {
    fn from(s: String) -> AccountId {
        s.parse().unwrap()
    }
}
