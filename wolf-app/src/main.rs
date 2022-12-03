use axum::{
    extract::{Json, Path, State, TypedHeader},
    headers::authorization::{Authorization, Basic},
    http::{Request, StatusCode},
    middleware::{map_request_with_state},
    routing::get,
    Router,
};
use axum_extra::routing::SpaRouter;
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};

pub mod app;
use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = match App::new("conf.json", "users.json") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to initialize app: err {}", e);
            std::process::exit(1);
        }
    };
    let d = Arc::new(RwLock::new(data));

    let app = Router::new()
        .route("/api/*path", get(api_get).patch(api_patch))
        .route("/api/", get(api_root_get).patch(api_root_patch))
        //.route_layer(middleware::from_fn_with_state(d1, auth_handler))
        .route_layer(map_request_with_state(d.clone(), auth_middleware))
        .with_state(d)
        .merge(SpaRouter::new("/", "ui").index_file("index.html"));

    axum::Server::bind(&"0.0.0.0:5080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn api_root_patch(
    State(app): State<Arc<RwLock<App>>>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    api_merge_data(app, "".to_owned(), payload)
}

async fn api_patch(
    State(app): State<Arc<RwLock<App>>>,
    Path(path): Path<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    api_merge_data(app, path, payload)
}

fn api_merge_data(app: Arc<RwLock<App>>, path: String, payload: Value) -> Json<Value> {
    let mut app = app.write().unwrap();

    let d = match app
        .data
        .pointer_mut(app::path_to_pointer(path.clone()).as_str())
    {
        Some(v) => v,
        None => {
            return Json(json!({
                "error": format!("path {} is not accessible", path)
            }))
        }
    };

    app::json_merge(d, payload);

    Json(json!("ok"))
}

async fn api_root_get(State(app): State<Arc<RwLock<App>>>) -> Json<Value> {
    Json(api_query(app, "".to_owned()))
}

async fn api_get(State(app): State<Arc<RwLock<App>>>, Path(path): Path<String>) -> Json<Value> {
    Json(api_query(app, path))
}

fn api_query(app: Arc<RwLock<App>>, path: String) -> Value {
    let app = app.read().unwrap();

    app.data
        .pointer(app::path_to_pointer(path).as_str())
        .unwrap_or(&json!(null))
        .clone()
}

async fn auth_middleware<B>(
    State(state): State<Arc<RwLock<App>>>,
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    request: Request<B>,
) -> Result<Request<B>, StatusCode> {
    //) -> Result<Response, StatusCode> {
    let app = state.read().unwrap();
    if !app.users.is_user_valid(auth.username(), auth.password()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(request)
}
