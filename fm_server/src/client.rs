use anyhow::Result;

#[derive(Clone)]
pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[macro_export]
macro_rules! client_post_macro {
    ( $url:expr, $path:expr, $x:expr ) => {{
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/{}", $url, $path))
            .body(serde_json::json!($x).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        Ok(serde_json::from_str(&response.text().await.unwrap())?)
    }};
}

impl fm_core::FinanceManager for Client {
    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<fm_core::account::AssetAccount> {
        client_post_macro!(self.url, "create_asset_account", (name, note, iban, bic))
    }

    async fn update_asset_account(
        &mut self,
        id: fm_core::Id,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<fm_core::account::AssetAccount> {
        client_post_macro!(
            self.url,
            "update_asset_account",
            (id, name, note, iban, bic)
        )
    }

    async fn get_accounts(&self) -> Result<Vec<fm_core::account::Account>> {
        let response = reqwest::get(&format!("{}/get_accounts", self.url))
            .await
            .unwrap();
        Ok(serde_json::from_str(&response.text().await?)?)
    }

    async fn get_account(&self, id: fm_core::Id) -> Result<Option<fm_core::account::Account>> {
        client_post_macro!(self.url, "get_account", id)
    }

    async fn get_account_sum(
        &self,
        account: &fm_core::account::Account,
        date: fm_core::DateTime,
    ) -> Result<fm_core::Currency> {
        client_post_macro!(self.url, "get_account_sum", (account, date))
    }

    async fn get_transaction(&self, id: fm_core::Id) -> Result<Option<fm_core::Transaction>> {
        client_post_macro!(self.url, "get_transaction", id)
    }

    async fn get_transactions_of_account(
        &self,
        account: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(self.url, "get_transactions_of_account", (account, timespan))
    }

    async fn create_transaction(
        &mut self,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Or<fm_core::Id, String>,
        destination: fm_core::Or<fm_core::Id, String>,
        budget: Option<fm_core::Id>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: Vec<fm_core::Id>,
    ) -> Result<fm_core::Transaction> {
        client_post_macro!(
            self.url,
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

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> Result<fm_core::account::BookCheckingAccount> {
        client_post_macro!(
            self.url,
            "create_book_checking_account",
            (name, notes, iban, bic)
        )
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::Recouring,
    ) -> Result<fm_core::Budget> {
        client_post_macro!(
            self.url,
            "create_budget",
            (name, description, total_value, timespan)
        )
    }

    async fn get_budgets(&self) -> Result<Vec<fm_core::Budget>> {
        let response = reqwest::get(&format!("{}/get_budgets", self.url))
            .await
            .unwrap();
        Ok(serde_json::from_str(&response.text().await?)?)
    }

    async fn get_transactions_of_budget(
        &self,
        budget: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(self.url, "get_transactions_of_budget", (budget, timespan))
    }

    async fn get_budget(&self, id: fm_core::Id) -> Result<Option<fm_core::Budget>> {
        client_post_macro!(self.url, "get_budget", id)
    }

    async fn update_transaction(
        &mut self,
        id: fm_core::Id,
        amount: fm_core::Currency,
        title: String,
        description: Option<String>,
        source: fm_core::Or<fm_core::Id, String>,
        destination: fm_core::Or<fm_core::Id, String>,
        budget: Option<fm_core::Id>,
        date: fm_core::DateTime,
        metadata: std::collections::HashMap<String, String>,
        categories: Vec<fm_core::Id>,
    ) -> Result<fm_core::Transaction> {
        client_post_macro!(
            self.url,
            "update_transaction",
            (
                id,
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

    async fn delete_transaction(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, "delete_transaction", id)
    }

    async fn update_budget(
        &mut self,
        id: fm_core::Id,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::Recouring,
    ) -> Result<fm_core::Budget> {
        client_post_macro!(
            self.url,
            "update_budget",
            (id, name, description, total_value, timespan)
        )
    }

    async fn get_transactions(
        &self,
        timespan: fm_core::Timespan,
    ) -> Result<Vec<fm_core::Transaction>> {
        client_post_macro!(self.url, "get_transactions", timespan)
    }

    async fn get_categories(&self) -> Result<Vec<fm_core::Category>> {
        let response = reqwest::get(&format!("{}/get_categories", self.url))
            .await
            .unwrap();
        Ok(serde_json::from_str(&response.text().await?)?)
    }

    async fn create_category(&mut self, name: String) -> Result<fm_core::Category> {
        client_post_macro!(self.url, "create_category", name)
    }

    async fn update_category(
        &mut self,
        id: fm_core::Id,
        name: String,
    ) -> Result<fm_core::Category> {
        client_post_macro!(self.url, "update_category", (id, name))
    }

    async fn get_category(&self, id: fm_core::Id) -> Result<Option<fm_core::Category>> {
        client_post_macro!(self.url, "get_category", id)
    }

    async fn delete_category(&mut self, id: fm_core::Id) -> Result<()> {
        client_post_macro!(self.url, "delete_category", id)
    }
}
