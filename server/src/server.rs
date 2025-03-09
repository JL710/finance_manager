use axum::{Router, response::Json, routing::get, routing::post};
use serde_json::{Value, json};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Tokenized;
use fm_core::FinanceManager;

#[derive(Clone)]
struct State {
    finance_controller: Arc<
        Mutex<
            fm_core::FMController<fm_core::managers::sqlite_finange_manager::SqliteFinanceManager>,
        >,
    >,
    token: String,
    timeout: Arc<Mutex<HashMap<std::net::IpAddr, Vec<u64>>>>,
}

fn timeout(map: &mut HashMap<std::net::IpAddr, Vec<u64>>, addr: &std::net::IpAddr) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if let Some(entry) = map.get_mut(addr) {
        entry.push(now);
        if entry.len() > 5 {
            entry.remove(0);
        }
    } else {
        let mut failed_list = Vec::with_capacity(5);
        failed_list.push(now);
        map.insert(*addr, failed_list);
    }
}

fn is_timeouted(map: &HashMap<std::net::IpAddr, Vec<u64>>, addr: &std::net::IpAddr) -> bool {
    const TIMEOUT: u64 = 60;

    if let Some(entry) = map.get(addr) {
        if let Some(last) = entry.last() {
            if std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - last
                > TIMEOUT
            {
                return false;
            }
        }
        if entry.len() == 5 {
            let mut diff = 0;
            for i in 1..entry.len() {
                diff += entry[i] - entry[i - 1];
            }
            if diff < TIMEOUT {
                return true;
            }
        }
    }
    false
}

async fn auth(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    if request.method() != axum::http::Method::POST {
        return Err(axum::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    // get request body
    let (parts, body) = request.into_parts();
    let body_data = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(data) => data,
        Err(_) => return Err(axum::http::StatusCode::BAD_REQUEST),
    };

    // extract token from body
    let tokenized: Tokenized<serde_json::Value> = match serde_json::from_slice(&body_data) {
        Ok(tokenized) => tokenized,
        Err(_) => return Err(axum::http::StatusCode::BAD_REQUEST),
    };

    let request = axum::http::Request::from_parts(
        parts,
        axum::body::Body::from(tokenized.content.to_string().into_bytes()),
    );

    if is_timeouted(&*state.timeout.lock().await, &addr.ip()) {
        tracing::info!("request timeouted");
        return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    if tokenized.token == state.token {
        let response = next.run(request).await;
        Ok(response)
    } else {
        timeout(&mut *state.timeout.lock().await, &addr.ip());
        tracing::info!("request unauthorized");
        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

pub fn init_subscriber() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .parse("tower_http=trace,fm_server=trace")
                .unwrap(),
        )
        .init();
}

pub async fn run_with_url(url: String, db: Option<String>, token: String) {
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    run_with_listener(listener, db, token).await;
}

pub async fn run_with_listener(
    listener: tokio::net::TcpListener,
    db: Option<String>,
    token: String,
) {
    let state = State {
        finance_controller: Arc::new(Mutex::new(if let Some(db_path) = db {
            fm_core::FMController::new(db_path).unwrap()
        } else {
            fm_core::FMController::with_finance_manager(
                fm_core::managers::SqliteFinanceManager::new_in_memory().unwrap(),
            )
        })),
        token,
        timeout: Arc::new(Mutex::new(HashMap::new())),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/get_budgets", post(get_budgets))
        .route(
            "/get_transactions_of_budget",
            post(get_transactions_of_budget),
        )
        .route("/get_accounts", post(get_accounts))
        .route("/create_asset_account", post(create_asset_account))
        .route("/get_account_sum", post(get_account_sum))
        .route("/get_account", post(get_account))
        .route(
            "/get_transactions_of_account",
            post(get_transactions_of_account),
        )
        .route("/create_budget", post(create_budget))
        .route("/delete_budget", post(delete_budget))
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
        .route(
            "/get_transactions_in_timespan",
            post(get_transactions_in_timespan),
        )
        .route("/get_transactions", post(get_transactions))
        .route("/get_categories", post(get_categories))
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
        .route("/get_bills", post(get_bills))
        .route("/get_bill", post(get_bill))
        .route("/delete_account", post(delete_account))
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth))
        .route("/status", get(status))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower::ServiceBuilder::new().layer(tower_http::trace::TraceLayer::new_for_http()))
        .with_state(state);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn status() -> String {
    String::from("Online")
}

async fn get_budgets(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let budgets = state
        .finance_controller
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
        .finance_controller
        .lock()
        .await
        .get_transactions_of_budget(data.0, data.1)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn get_accounts(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let accounts = state
        .finance_controller
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
        Option<fm_core::AccountId>,
        Option<String>,
        fm_core::Currency,
    )>,
) -> Json<Value> {
    let account = state
        .finance_controller
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
        .finance_controller
        .lock()
        .await
        .raw_fm()
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
        .finance_controller
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
        .finance_controller
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
        fm_core::Recurring,
    )>,
) -> Json<Value> {
    let budget = state
        .finance_controller
        .lock()
        .await
        .create_budget(data.0, data.1, data.2, data.3)
        .await
        .unwrap();
    json!(budget).into()
}

async fn delete_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_controller
        .lock()
        .await
        .delete_budget(data)
        .await
        .unwrap();
    json!(()).into()
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
        HashMap<fm_core::Id, fm_core::Sign>,
    )>,
) -> Json<Value> {
    let transaction = state
        .finance_controller
        .lock()
        .await
        .create_transaction(
            data.0, data.1, data.2, data.3, data.4, data.5, data.6, data.7, data.8,
        )
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
        Option<fm_core::AccountId>,
        Option<String>,
    )>,
) -> Json<Value> {
    let account = state
        .finance_controller
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
        .finance_controller
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
        Option<fm_core::AccountId>,
        Option<String>,
        fm_core::Currency,
    )>,
) -> Json<Value> {
    let account = state
        .finance_controller
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
        .finance_controller
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
    axum::extract::Json(data): axum::extract::Json<fm_core::Transaction>,
) -> Json<Value> {
    let transaction = state
        .finance_controller
        .lock()
        .await
        .update_transaction(data)
        .await
        .unwrap();
    json!(transaction).into()
}

async fn delete_transaction(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_controller
        .lock()
        .await
        .delete_transaction(data)
        .await
        .unwrap();
    json!(()).into()
}

async fn update_budget(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Budget>,
) -> Json<Value> {
    let budget = state
        .finance_controller
        .lock()
        .await
        .update_budget(data)
        .await
        .unwrap();
    json!(budget).into()
}

async fn get_transactions_in_timespan(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Timespan>,
) -> Json<Value> {
    let transactions = state
        .finance_controller
        .lock()
        .await
        .get_transactions_in_timespan(data)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn get_transactions(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<Vec<fm_core::Id>>,
) -> Json<Value> {
    let transactions = state
        .finance_controller
        .lock()
        .await
        .get_transactions(data)
        .await
        .unwrap();
    json!(transactions).into()
}

async fn get_categories(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let categories = state
        .finance_controller
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
        .finance_controller
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
        .finance_controller
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
        .finance_controller
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
        .finance_controller
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
        .finance_controller
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
        Option<fm_core::AccountId>,
        Option<String>,
    )>,
) -> Json<Value> {
    let account = state
        .finance_controller
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
        .finance_controller
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
        HashMap<fm_core::Id, fm_core::Sign>,
        Option<fm_core::DateTime>,
    )>,
) -> Json<Value> {
    let bill = state
        .finance_controller
        .lock()
        .await
        .create_bill(data.0, data.1, data.2, data.3, data.4)
        .await
        .unwrap();
    json!(bill).into()
}

async fn delete_bill(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_controller
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
    axum::extract::Json(data): axum::extract::Json<fm_core::Bill>,
) -> Json<Value> {
    state
        .finance_controller
        .lock()
        .await
        .update_bill(data)
        .await
        .unwrap();
    json!(()).into()
}

async fn get_bills(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let bills = state
        .finance_controller
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
        .finance_controller
        .lock()
        .await
        .get_bill(&data)
        .await
        .unwrap();
    json!(bill).into()
}

async fn delete_account(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Json(data): axum::extract::Json<fm_core::Id>,
) -> Json<Value> {
    state
        .finance_controller
        .lock()
        .await
        .delete_account(data, false)
        .await
        .unwrap();
    json!(()).into()
}
