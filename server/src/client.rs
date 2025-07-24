use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Client {
    url: String,
    token: String,
}

impl Client {
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[macro_export]
macro_rules! client_post_macro {
    ( $url:expr, $token:expr, $path:expr, $x:expr ) => {{
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/{}", $url, $path))
            .body(
                serde_json::json!($crate::Tokenized {
                    token: $token,
                    content: $x
                })
                .to_string(),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .send()
            .await
            .unwrap();
        Ok(serde_json::from_str(&response.text().await.unwrap())?)
    }};
}

impl fm_core::FinanceManager for Client {
    type Flags = (String, String);

    fn new(flags: Self::Flags) -> Result<Self> {
        Ok(Self {
            url: flags.0,
            token: flags.1,
        })
    }

    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<fm_core::AccountId>,
        bic: Option<fm_core::Bic>,
        offset: fm_core::Currency,
    ) -> Result<fm_core::account::AssetAccount> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "create_asset_account",
            (name, note, iban, bic, offset)
        )
    }

    async fn update_asset_account(
        &mut self,
        account: fm_core::account::AssetAccount,
    ) -> Result<fm_core::account::AssetAccount> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "update_asset_account",
            account
        )
    }

    async fn delete_account(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "delete_account", id)
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<fm_core::AccountId>,
        bic: Option<fm_core::Bic>,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "create_book_checking_account",
            (name, notes, iban, bic)
        )
    }

    async fn update_book_checking_account(
        &mut self,
        account: fm_core::account::BookCheckingAccount,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "update_book_checking_account",
            account
        )
    }

    async fn get_account_sum(
        &self,
        account: &fm_core::account::Account,
        date: fm_core::DateTime,
    ) -> Result<fm_core::Currency> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_account_sum",
            (account, date)
        )
    }

    async fn create_bill(
        &mut self,
        name: String,
        description: Option<String>,
        value: fm_core::Currency,
        transactions: HashMap<fm_core::Id, fm_core::Sign>,
        due_date: Option<fm_core::DateTime>,
        closed: bool,
    ) -> Result<fm_core::Bill> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "create_bill",
            (name, description, value, transactions, due_date, closed)
        )
    }

    async fn update_bill(&mut self, bill: fm_core::Bill) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "update_bill", bill)
    }

    async fn get_bills(&self, closed: Option<bool>) -> Result<Vec<fm_core::Bill>> {
        client_post_macro!(self.url, self.token.clone(), "get_bills", closed)
    }

    async fn get_bill(&self, id: &fm_core::Id) -> Result<Option<fm_core::Bill>> {
        client_post_macro!(self.url, self.token.clone(), "get_bill", id)
    }

    async fn delete_bill(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "delete_bill", id)
    }

    async fn get_filtered_transactions(
        &self,
        filter: fm_core::transaction_filter::TransactionFilter,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_filtered_transactions",
            filter
        )
    }

    async fn get_accounts(&self) -> Result<Vec<fm_core::account::Account>> {
        client_post_macro!(self.url, self.token.clone(), "get_accounts", ())
    }

    async fn get_account(&self, id: fm_core::Id) -> Result<Option<fm_core::account::Account>> {
        client_post_macro!(self.url, self.token.clone(), "get_account", id)
    }

    async fn get_transaction(&self, id: fm_core::Id) -> Result<Option<fm_core::Transaction>> {
        client_post_macro!(self.url, self.token.clone(), "get_transaction", id)
    }

    async fn get_transactions_of_account(
        &self,
        account: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_transactions_of_account",
            (account, timespan)
        )
    }

    async fn create_transaction(
        &mut self,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Id,
        destination: fm_core::Id,
        budget: Option<(fm_core::Id, fm_core::Sign)>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: HashMap<fm_core::Id, fm_core::Sign>,
    ) -> Result<fm_core::Transaction> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "create_transaction",
            (
                amount,
                title,
                description,
                source,
                destination,
                budget,
                date,
                metadata,
                categories
            )
        )
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::budget::Recurring,
    ) -> Result<fm_core::Budget> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "create_budget",
            (name, description, total_value, timespan)
        )
    }

    async fn delete_budget(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "delete_budget", id)
    }

    async fn get_budgets(&self) -> Result<Vec<fm_core::Budget>> {
        client_post_macro!(self.url, self.token.clone(), "get_budgets", ())
    }

    async fn get_transactions_of_budget(
        &self,
        budget: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_transactions_of_budget",
            (budget, timespan)
        )
    }

    async fn get_budget(&self, id: fm_core::Id) -> Result<Option<fm_core::Budget>> {
        client_post_macro!(self.url, self.token.clone(), "get_budget", id)
    }

    async fn update_transaction(
        &mut self,
        transaction: fm_core::Transaction,
    ) -> Result<fm_core::Transaction> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "update_transaction",
            transaction
        )
    }

    async fn delete_transaction(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "delete_transaction", id)
    }

    async fn update_budget(&mut self, budget: fm_core::Budget) -> Result<fm_core::Budget> {
        client_post_macro!(self.url, self.token.clone(), "update_budget", budget)
    }

    async fn get_transactions_in_timespan(
        &self,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_transactions_in_timespan",
            timespan
        )
    }

    async fn get_categories(&self) -> Result<Vec<fm_core::Category>> {
        client_post_macro!(self.url, self.token.clone(), "get_categories", ())
    }

    async fn create_category(&mut self, name: String) -> Result<fm_core::Category> {
        client_post_macro!(self.url, self.token.clone(), "create_category", name)
    }

    async fn update_category(&mut self, category: fm_core::Category) -> Result<fm_core::Category> {
        client_post_macro!(self.url, self.token.clone(), "update_category", category)
    }

    async fn get_category(&self, id: fm_core::Id) -> Result<Option<fm_core::Category>> {
        client_post_macro!(self.url, self.token.clone(), "get_category", id)
    }

    async fn delete_category(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, self.token.clone(), "delete_category", id)
    }

    async fn get_transactions_of_category(
        &self,
        category: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(
            self.url,
            self.token.clone(),
            "get_transactions_of_category",
            (category, timespan)
        )
    }

    async fn get_transactions(&self, ids: Vec<fm_core::Id>) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(self.url, self.token.clone(), "get_transactions", ids)
    }
}
