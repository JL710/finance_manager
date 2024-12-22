use std::ops::Neg;

use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Hash)]
pub enum Currency {
    Eur(BigDecimal),
}

impl Currency {
    pub fn to_num_string(&self) -> String {
        match self {
            Currency::Eur(num) => num.to_string(),
        }
    }

    pub fn get_eur_num(&self) -> f64 {
        match self {
            Currency::Eur(x) => x.to_f64().unwrap(),
        }
    }

    pub fn get_currency_id(&self) -> i32 {
        match self {
            Currency::Eur(_) => 1,
        }
    }

    pub fn from_currency_id(id: i32, amount: BigDecimal) -> Result<Self> {
        match id {
            1 => Ok(Currency::Eur(amount.round(2))),
            _ => anyhow::bail!("not a valid currency id"),
        }
    }

    pub fn negative(&self) -> Self {
        match self {
            Currency::Eur(x) => Currency::Eur(x.neg()),
        }
    }
}

impl PartialOrd for Currency {
    fn ge(&self, other: &Self) -> bool {
        self.get_eur_num() >= other.get_eur_num()
    }

    fn gt(&self, other: &Self) -> bool {
        self.get_eur_num() > other.get_eur_num()
    }

    fn le(&self, other: &Self) -> bool {
        self.get_eur_num() <= other.get_eur_num()
    }

    fn lt(&self, other: &Self) -> bool {
        self.get_eur_num() < other.get_eur_num()
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Currency {}

impl Ord for Currency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_eur_num().total_cmp(&other.get_eur_num())
    }
}

fn big_decimal_to_string(decimal: &BigDecimal) -> String {
    let decimal_string = decimal.to_string();
    let splits = decimal_string.split('.').collect::<Vec<_>>();

    let mut pre_decimal_part = String::new();
    for c in splits[0].chars().rev().enumerate() {
        if c.0 % 3 == 0 {
            pre_decimal_part += " ";
        }
        pre_decimal_part += &c.1.to_string();
    }
    pre_decimal_part = pre_decimal_part.chars().rev().collect();

    if splits.len() == 1 {
        pre_decimal_part
    } else if splits.len() == 2 {
        format!("{}.{}", pre_decimal_part, splits[1])
    } else {
        panic!("to many parts of number string")
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::Eur(value) => write!(f, "{}€", big_decimal_to_string(value)),
        }
    }
}

impl std::ops::Add for Currency {
    type Output = Currency;

    fn add(self, other: Currency) -> Self::Output {
        match self {
            Currency::Eur(value) => match other {
                Currency::Eur(other_value) => Currency::Eur(value + other_value),
            },
        }
    }
}

impl std::ops::Sub for Currency {
    type Output = Currency;

    fn sub(self, other: Currency) -> Self::Output {
        match self {
            Currency::Eur(value) => match other {
                Currency::Eur(other_value) => Currency::Eur(value - other_value),
            },
        }
    }
}

impl std::ops::AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl std::ops::SubAssign for Currency {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

impl From<f64> for Currency {
    fn from(value: f64) -> Self {
        Currency::Eur(BigDecimal::from_f64(value).unwrap())
    }
}

impl Default for Currency {
    /// Creates a new `Currency` with a EUR value of 0.0.
    fn default() -> Self {
        Currency::Eur(BigDecimal::from_f64(0.0).unwrap())
    }
}
