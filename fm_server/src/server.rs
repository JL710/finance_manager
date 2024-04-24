use axum::{response::Json, routing::get, routing::post, Router};
use serde_json::{json, Value};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use fm_core::FinanceManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct State {
    finance_manager: Arc<Mutex<fm_core::ram_finance_manager::RamFinanceManager>>,
}

#[tokio::main]
pub async fn run() {
    let state = State {
        finance_manager: Arc::new(Mutex::new(
            fm_core::ram_finance_manager::RamFinanceManager::new(),
        )),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .parse("tower_http=trace,fm_server=trace")
                .unwrap(),
        )
        .init();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/get_budgets", get(get_budgets))
        .route(
            "/get_transactions_of_budget",
            post(get_transactions_of_budget),
        )
        .route("/get_accounts", get(get_accounts))
        .route("/create_asset_account", post(create_asset_account))
        .route("/get_account_sum", post(get_account_sum))
        .route("/get_account", post(get_account))
        .route(
            "/get_transactions_of_account",
            post(get_transactions_of_account),
        )
        .route("/create_budget", post(create_budget))
        .route("/create_transaction", post(create_transaction))
        .route(
            "/create_book_checking_account",
            post(create_book_checking_account),
        )
        .route("/get_transaction", post(get_transaction))
        .route("/update_asset_account", post(update_asset_account))
        .layer(tower::ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_budgets(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let budgets = state
        .finance_manager
        .lock()
        .await
        .get_budgets()
        .await
        .unwrap();
    json!(budgets).into()
}

async fn get_transactions_of_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(budget): axum::extract::Json<fm_core::Budget>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_transactions_of_budget(&budget)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn get_accounts(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let accounts = state
        .finance_manager
        .lock()
        .await
        .get_accounts()
        .await
        .unwrap();
    json!(accounts).into()
}

async fn create_asset_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(account_data): axum::extract::Json<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
) -> Json<Value> {
    let account = state
        .finance_manager
        .lock()
        .await
        .create_asset_account(
            account_data.0,
            account_data.1,
            account_data.2,
            account_data.3,
        )
        .await
        .unwrap();
    println!("Created account: {:?}", account);
    json!(account).into()
}

async fn get_account_sum(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(account_data): axum::extract::Json<(
        fm_core::account::Account,
        fm_core::DateTime,
    )>,
) -> Json<Value> {
    let sum = state
        .finance_manager
        .lock()
        .await
        .get_account_sum(&account_data.0, account_data.1)
        .await
        .unwrap();
    json!(sum).into()
}

async fn get_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(id): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    let account = state
        .finance_manager
        .lock()
        .await
        .get_account(id)
        .await
        .unwrap();
    json!(account).into()
}

async fn get_transactions_of_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(fm_core::Id, fm_core::Timespan)>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_transactions_of_account(data.0, data.1)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn create_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        String,
        Option<String>,
        fm_core::Currency,
        fm_core::Recouring,
    )>,
) -> Json<Value> {
    let budget = state
        .finance_manager
        .lock()
        .await
        .create_budget(data.0, data.1, data.2, data.3)
        .await
        .unwrap();
    json!(budget).into()
}

async fn create_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Currency,
        String,
        Option<String>,
        fm_core::Or<fm_core::Id, String>,
        fm_core::Or<fm_core::Id, String>,
        Option<fm_core::Id>,
        fm_core::DateTime,
    )>,
) -> Json<Value> {
    let transaction = state
        .finance_manager
        .lock()
        .await
        .create_transaction(data.0, data.1, data.2, data.3, data.4, data.5, data.6)
        .await
        .unwrap();
    json!(transaction).into()
}

async fn create_book_checking_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
) -> Json<Value> {
    let account = state
        .finance_manager
        .lock()
        .await
        .create_book_checking_account(data.0, data.1, data.2, data.3)
        .await
        .unwrap();
    json!(account).into()
}

async fn get_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    let transaction = state
        .finance_manager
        .lock()
        .await
        .get_transaction(data)
        .await
        .unwrap();
    json!(transaction).into()
}

async fn update_asset_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Id,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
) -> Json<Value> {
    let account = state
        .finance_manager
        .lock()
        .await
        .update_asset_account(data.0, data.1, data.2, data.3, data.4)
        .await
        .unwrap();
    json!(account).into()
}
