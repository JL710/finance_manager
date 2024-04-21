#[derive(Clone)]
pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl fm_core::FinanceManager for Client {
    async fn create_asset_account(
        &mut self,
        name: String,
        note: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> fm_core::account::AssetAccount {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/create_asset_account", self.url))
            .body(serde_json::json!((name, note, iban, bic)).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn get_accounts(&self) -> Vec<fm_core::account::Account> {
        let response = reqwest::get(&format!("{}/get_accounts", self.url))
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn get_account(&self, id: fm_core::Id) -> Option<fm_core::account::Account> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/get_account", self.url))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::json!(id).to_string())
            .send()
            .await
            .unwrap();

        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn get_account_sum(
        &self,
        account: &fm_core::account::Account,
        date: fm_core::DateTime,
    ) -> fm_core::Currency {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/get_account_sum", &self.url))
            .body(serde_json::json!((account, date)).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn get_transaction(&self, id: fm_core::Id) -> Option<fm_core::Transaction> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/get_transaction", &self.url))
            .body(serde_json::json!(id).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn get_transactions_of_account(
        &self,
        account: fm_core::Id,
        timespan: fm_core::Timespan,
    ) -> Vec<fm_core::Transaction> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/get_transactions_of_account", &self.url))
            .body(serde_json::json!((account, timespan)).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
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
    ) -> fm_core::Transaction {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/create_transaction", &self.url))
            .body(
                serde_json::json!((
                    amount,
                    title,
                    description,
                    source,
                    destination,
                    budget,
                    date
                ))
                .to_string(),
            )
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn create_book_checking_account(
        &mut self,
        name: String,
        notes: Option<String>,
        iban: Option<String>,
        bic: Option<String>,
    ) -> fm_core::account::BookCheckingAccount {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/create_book_checking_account", &self.url))
            .body(serde_json::json!((name, notes, iban, bic,)).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        serde_json::from_str(&response.text().await.unwrap()).unwrap()
    }

    async fn create_budget(
        &mut self,
        name: String,
        description: Option<String>,
        total_value: fm_core::Currency,
        timespan: fm_core::Recouring,
    ) -> fm_core::Budget {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/create_budget", self.url))
            .body(serde_json::json!((name, description, total_value, timespan)).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        return serde_json::from_str(&response.text().await.unwrap()).unwrap();
    }

    async fn get_budgets(&self) -> Vec<fm_core::Budget> {
        let response = reqwest::get(&format!("{}/get_budgets", self.url))
            .await
            .unwrap();
        return serde_json::from_str(&response.text().await.unwrap()).unwrap();
    }

    async fn get_transactions_of_budget(
        &self,
        budget: &fm_core::Budget,
    ) -> Vec<fm_core::Transaction> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/get_transactions_of_budget", self.url))
            .body(serde_json::json!(budget).to_string())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
        return serde_json::from_str(&response.text().await.unwrap()).unwrap();
    }
}
