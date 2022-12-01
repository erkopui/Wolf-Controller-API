use axum::{
    extract::{Json, Path, State},
    routing::get,
    Router,
};
use axum_extra::routing::SpaRouter;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub mod data;
use data::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = match App::new("conf.json") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to initialize app: err {}", e);
            std::process::exit(1);
        }
    };
    let data = Arc::new(Mutex::new(data));

    let app = Router::new()
        .route("/api/*path", get(api_get))
        .route("/api/", get(api_root_get))
        .with_state(data)
        .merge(SpaRouter::new("/", "ui").index_file("index.html"));

    axum::Server::bind(&"0.0.0.0:5080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn api_root_get(State(app): State<Arc<Mutex<App>>>) -> Json<Value> {
    Json(api_query(app, None))
}

async fn api_get(State(app): State<Arc<Mutex<App>>>, Path(path): Path<String>) -> Json<Value> {
    Json(api_query(app, Some(path)))
}

fn api_query(app: Arc<Mutex<App>>, path: Option<String>) -> Value {
    let app = app.lock().unwrap();

    match path {
        None => return app.data.clone(),
        Some(mut v) => {
            if v.ends_with("/") {
                v.pop();
            }

            let keys: Vec<&str> = v.split('/').collect();
            let mut data = &app.data;
            for i in keys {
                data = &data[i];
            }

            return data.clone();
        }
    }
}
