use axum::{
    extract::{DefaultBodyLimit, Json, Multipart, Path, State, TypedHeader},
    headers::authorization::{Authorization, Basic},
    http::{Request, StatusCode},
    middleware::map_request_with_state,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Router,
};
use axum_extra::routing::SpaRouter;
use core::time::Duration;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};

use tower_http::limit::RequestBodyLimitLayer;

pub mod app;
use app::{user, App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = match App::new("conf.json") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to initialize app: err {}", e);
            std::process::exit(1);
        }
    };

    let d = Arc::new(RwLock::new(app));
    let gpio_app = d.clone();
    let adc_app = d.clone();

    let _gpio_handle = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(50));
        let app = gpio_app.write().unwrap();
        if let Some(v) = &app.gpio {
            v.read().unwrap().poll(&app.store);
        } 
    });

    let _gpio_adc_handle = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(25));
        let app = adc_app.write().unwrap();
        if let Some(v) = &app.gpio {
            v.write().unwrap().adc_poll(&app.store);
        } 
    });

    let app = Router::new()
        .route("/api/*path", get(api_get).patch(api_patch))
        .route("/api/", get(api_root_get).patch(api_root_patch))
        .route("/api", get(api_root_get).patch(api_root_patch))
        .route("/user", get(api_user_get).put(api_user_put))
        .route("/firmware", post(firmware_upload))
        .route("/reboot", get(api_reboot))
        .route("/save", get(api_save))
        .route_layer(map_request_with_state(d.clone(), auth_middleware))
        .with_state(d)
        .merge(SpaRouter::new("/", "ui").index_file("index.html"))
        .layer(RequestBodyLimitLayer::new(
            16 * 1024 * 1024, /* 16 mb */
        ))
        .layer(DefaultBodyLimit::max(16 * 1024 * 1024));

    let addr = "0.0.0.0:5080".parse().unwrap();
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn firmware_upload(State(app): State<Arc<RwLock<App>>>, mut multipart: Multipart) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        //let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let app = app.read().unwrap();
        match app.firmware.save(data) {
            Err(e) => eprintln!("{}", e),
            Ok(_) => println!("Firmware file write was successful"),
        }
    }
}

async fn api_user_put(
    State(app): State<Arc<RwLock<App>>>,
    Json(user): Json<user::User>,
) -> Result<StatusCode, StatusCode> {
    let mut app = app.write().unwrap();
    match app.users.update(user) {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        _ => return Ok(StatusCode::NO_CONTENT),
    }
}

async fn api_user_get(State(app): State<Arc<RwLock<App>>>) -> Json<user::User> {
    let app = app.read().unwrap();
    let mut r: HashMap<String, user::UserData> = HashMap::new();
    for (p, v) in &app.users.users.user {
        let mut o = v.clone();
        o.password = "******".to_owned();
        r.insert(p.clone(), o);
    }
    Json(user::User { user: r })
}

async fn api_root_patch(
    State(app): State<Arc<RwLock<App>>>,
    Json(payload): Json<Value>,
) -> Response {
    api_merge_data(app, "".to_owned(), payload)
}

async fn api_patch(
    State(app): State<Arc<RwLock<App>>>,
    Path(path): Path<String>,
    Json(payload): Json<Value>,
) -> Response {
    api_merge_data(app, path, payload)
}

fn api_merge_data(app: Arc<RwLock<App>>, path: String, payload: Value) -> Response {
    let pointer = app::path_to_pointer(path.clone());

    {
        let a = &app.read().unwrap();
        let data = &a.store.read().unwrap();
        if let Err(e) = a.validator.validate(&pointer, &data, &payload, None, None) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("failed to validate request, msg: {}", e)
                })),
            )
                .into_response();
        }
    }

    if let Err(e) = App::merge_data(app, pointer, &payload) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("failed to write data, msg: {}", e)
            })),
        )
            .into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

async fn api_root_get(State(app): State<Arc<RwLock<App>>>) -> Json<Value> {
    Json(api_query(app, "".to_owned()))
}

async fn api_get(State(app): State<Arc<RwLock<App>>>, Path(path): Path<String>) -> Json<Value> {
    Json(api_query(app, path))
}

fn api_query(app: Arc<RwLock<App>>, path: String) -> Value {
    let app = app.read().unwrap();
    let data = &app.store.read().unwrap();

    data.pointer(app::path_to_pointer(path).as_str())
        .unwrap_or(&json!(null))
        .clone()
}

async fn api_reboot(State(app): State<Arc<RwLock<App>>>) {
    app.read().unwrap().reboot();
}

async fn api_save(State(app): State<Arc<RwLock<App>>>) -> Response {
    if let Err(e) = app.read().unwrap().save() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("failed to validate request, msg: {}", e)
            })),
        )
            .into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

async fn auth_middleware<B>(
    State(state): State<Arc<RwLock<App>>>,
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    request: Request<B>,
) -> Result<Request<B>, StatusCode> {
    let app = state.read().unwrap();
    if !app.users.is_user_valid(auth.username(), auth.password()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(request)
}
