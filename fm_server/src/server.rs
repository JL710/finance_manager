use axum::{response::Json, routing::get, routing::post, Router};
use serde_json::{json, Value};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct State {
    finance_manager: Arc<
        Mutex<
            fm_core::FMController<fm_core::managers::sqlite_finange_manager::SqliteFinanceManager>,
        >,
    >,
}

#[tokio::main]
pub async fn run(url: String, db: String) {
    let state = State {
        finance_manager: Arc::new(Mutex::new(fm_core::FMController::new(db).unwrap())),
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
        .route("/get_budget", post(get_budget))
        .route("/update_transaction", post(update_transaction))
        .route("/delete_transaction", post(delete_transaction))
        .route("/update_budget", post(update_budget))
        .route("/get_transactions", post(get_transactions))
        .route("/get_categories", get(get_categories))
        .route("/get_category", post(get_category))
        .route("/create_category", post(create_category))
        .route("/update_category", post(update_category))
        .route("/delete_category", post(delete_category))
        .route(
            "/get_transactions_of_category",
            post(get_transactions_of_category),
        )
        .route(
            "/update_book_checking_account",
            post(update_book_checking_account),
        )
        .route(
            "/get_filtered_transactions",
            post(get_filtered_transactions),
        )
        .route("/create_bill", post(create_bill))
        .route("/delete_bill", post(delete_bill))
        .route("/update_bill", post(update_bill))
        .route("/get_bills", get(get_bills))
        .route("/get_bill", post(get_bill))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower::ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
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
    axum::extract::Json(data): axum::extract::Json<(fm_core::Id, fm_core::Timespan)>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_transactions_of_budget(data.0, data.1)
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

#[allow(clippy::type_complexity)]
async fn create_asset_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(account_data): axum::extract::Json<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        fm_core::Currency,
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
            account_data.4,
        )
        .await
        .unwrap();
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

#[allow(clippy::type_complexity)]
async fn create_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Currency,
        String,
        Option<String>,
        fm_core::Id,
        fm_core::Id,
        Option<(fm_core::Id, fm_core::Sign)>,
        fm_core::DateTime,
        std::collections::HashMap<String, String>,
        Vec<(fm_core::Id, fm_core::Sign)>,
    )>,
) -> Json<Value> {
    let transaction = state
        .finance_manager
        .lock()
        .await
        .create_transaction(
            data.0, data.1, data.2, data.3, data.4, data.5, data.6, data.7, data.8,
        )
        .unwrap()
        .await
        .unwrap();
    json!(transaction).into()
}

#[allow(clippy::type_complexity)]
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

#[allow(clippy::type_complexity)]
async fn update_asset_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Id,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        fm_core::Currency,
    )>,
) -> Json<Value> {
    let account = state
        .finance_manager
        .lock()
        .await
        .update_asset_account(data.0, data.1, data.2, data.3, data.4, data.5)
        .await
        .unwrap();
    json!(account).into()
}

async fn get_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    let budget = state
        .finance_manager
        .lock()
        .await
        .get_budget(data)
        .await
        .unwrap();
    json!(budget).into()
}

#[allow(clippy::type_complexity)]
async fn update_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Id,
        fm_core::Currency,
        String,
        Option<String>,
        fm_core::Id,
        fm_core::Id,
        Option<(fm_core::Id, fm_core::Sign)>,
        fm_core::DateTime,
        std::collections::HashMap<String, String>,
        Vec<(fm_core::Id, fm_core::Sign)>,
    )>,
) -> Json<Value> {
    let transaction = state
        .finance_manager
        .lock()
        .await
        .update_transaction(
            data.0, data.1, data.2, data.3, data.4, data.5, data.6, data.7, data.8, data.9,
        )
        .unwrap()
        .await
        .unwrap();
    json!(transaction).into()
}

async fn delete_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_manager
        .lock()
        .await
        .delete_transaction(data)
        .await
        .unwrap();
    json!(()).into()
}

async fn update_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Id,
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
        .update_budget(data.0, data.1, data.2, data.3, data.4)
        .await
        .unwrap();
    json!(budget).into()
}

async fn get_transactions(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Timespan>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_transactions(data)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn get_categories(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let categories = state
        .finance_manager
        .lock()
        .await
        .get_categories()
        .await
        .unwrap();
    json!(categories).into()
}

async fn get_category(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    let category = state
        .finance_manager
        .lock()
        .await
        .get_category(data)
        .await
        .unwrap();
    json!(category).into()
}

async fn create_category(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<String>,
) -> Json<Value> {
    let category = state
        .finance_manager
        .lock()
        .await
        .create_category(data)
        .await
        .unwrap();
    json!(category).into()
}

async fn update_category(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(fm_core::Id, String)>,
) -> Json<Value> {
    let category = state
        .finance_manager
        .lock()
        .await
        .update_category(data.0, data.1)
        .await
        .unwrap();
    json!(category).into()
}

async fn delete_category(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_manager
        .lock()
        .await
        .delete_category(data)
        .await
        .unwrap();
    json!(()).into()
}

async fn get_transactions_of_category(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(fm_core::Id, fm_core::Timespan)>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_transactions_of_category(data.0, data.1)
        .await
        .unwrap();
    json!(transactions).into()
}

#[allow(clippy::type_complexity)]
async fn update_book_checking_account(
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
        .update_book_checking_account(data.0, data.1, data.2, data.3, data.4)
        .await
        .unwrap();
    json!(account).into()
}

async fn get_filtered_transactions(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::transaction_filter::TransactionFilter>,
) -> Json<Value> {
    let transactions = state
        .finance_manager
        .lock()
        .await
        .get_filtered_transactions(data)
        .await
        .unwrap();
    json!(transactions).into()
}

#[allow(clippy::type_complexity)]
async fn create_bill(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        String,
        Option<String>,
        fm_core::Currency,
        Vec<(fm_core::Id, fm_core::Sign)>,
        Option<fm_core::DateTime>,
    )>,
) -> Json<Value> {
    let bill = state
        .finance_manager
        .lock()
        .await
        .create_bill(data.0, data.1, data.2, data.3, data.4)
        .unwrap()
        .await
        .unwrap();
    json!(bill).into()
}

async fn delete_bill(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_manager
        .lock()
        .await
        .delete_bill(data)
        .await
        .unwrap();
    json!(()).into()
}

#[allow(clippy::type_complexity)]
async fn update_bill(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<(
        fm_core::Id,
        String,
        Option<String>,
        fm_core::Currency,
        Vec<(fm_core::Id, fm_core::Sign)>,
        Option<fm_core::DateTime>,
    )>,
) -> Json<Value> {
    state
        .finance_manager
        .lock()
        .await
        .update_bill(data.0, data.1, data.2, data.3, data.4, data.5)
        .unwrap()
        .await
        .unwrap();
    json!(()).into()
}

async fn get_bills(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let bills = state
        .finance_manager
        .lock()
        .await
        .get_bills()
        .await
        .unwrap();
    json!(bills).into()
}

async fn get_bill(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    let bill = state
        .finance_manager
        .lock()
        .await
        .get_bill(&data)
        .await
        .unwrap();
    json!(bill).into()
}
