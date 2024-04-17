use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

#[derive(Clone)]
struct State {
    finance_manager: fm_core::FinanceManager,
}

#[tokio::main]
pub async fn run() {
    let state = State {
        finance_manager: fm_core::FinanceManager::new(),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/budgets", get(get_budgets))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_budgets(axum::extract::State(state): axum::extract::State<State>) -> Json<Value> {
    let budgets = state.finance_manager.get_budgets();
    json!(budgets).into()
}
