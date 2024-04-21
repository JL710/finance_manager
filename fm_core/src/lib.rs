pub mod account;
pub mod ram_finance_manager;

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Id = u64;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum Currency {
    Eur(f64),
}

impl Currency {
    pub fn to_num_string(&self) -> String {
        match self {
            Currency::Eur(num) => num.to_string(),
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::Eur(value) => write!(f, "{}€", value),
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

impl std::ops::AddAssign for Currency {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    id: Id,
    name: String,
    description: Option<String>,
    total_value: Currency,
    timespan: Recouring,
}

impl Budget {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(desc) => Some(desc),
            None => None,
        }
    }

    pub fn total_value(&self) -> Currency {
        self.total_value.clone()
    }

    pub fn timespan(&self) -> &Recouring {
        &self.timespan
    }

    pub fn id(&self) -> &Id {
        &self.id
    }
}

impl std::fmt::Display for Budget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    id: Id,
    amount: Currency, // if amount is positive the money is added to the account. If negative it is removed
    title: String,
    description: Option<String>,
    source: Id,
    destination: Id,
    budget: Option<Id>,
    date: DateTime,
}

impl Transaction {
    fn connection_with_account(&self, account: Id) -> bool {
        if account == self.source {
            return true;
        }
        if account == self.destination {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Recouring {
    Days(DateTime, usize), // start time and days
    DayInMonth(u16),       // i.e. 3. of each month
    Yearly(u8, u16),       // month and day
}

pub type Timespan = (Option<DateTime>, Option<DateTime>);

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Or<T, F> {
    One(T),
    Two(F),
}

pub trait FinanceManager: Send + Clone + Sized {
    fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = account::AssetAccount> + Send;

    fn get_accounts(&self) -> impl futures::Future<Output = Vec<account::Account>> + Send;

    fn get_account(&self, id: Id)
        -> impl futures::Future<Output = Option<account::Account>> + Send;

    fn get_account_sum(
        &self,
        account: &account::Account,
        date: DateTime,
    ) -> impl futures::Future<Output = Currency> + Send;

    fn get_transaction(&self, id: Id) -> impl futures::Future<Output = Option<Transaction>> + Send;

    fn get_transactions_of_account(
        &self,
        account: Id,
        timespan: Timespan,
    ) -> impl futures::Future<Output = Vec<Transaction>> + Send;

    fn create_transaction(
        &mut self,
        amount: Currency,
        title: String,
        description: Option<String>,
        source: Or<Id, String>, // id = Existing | String = New
        destination: Or<Id, String>,
        budget: Option<Id>,
        date: DateTime,
    ) -> impl futures::Future<Output = Transaction> + Send;

    fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> impl futures::Future<Output = account::BookCheckingAccount> + Send;

    fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: Currency,
        timespan: Recouring,
    ) -> impl futures::Future<Output = Budget> + Send;

    fn get_budgets(&self) -> impl futures::Future<Output = Vec<Budget>> + Send;

    fn get_transactions_of_budget(
        &self,
        budget: &Budget,
    ) -> impl futures::Future<Output = Vec<Transaction>> + Send;
}
