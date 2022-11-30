use axum::{Router};
use axum_extra::routing::SpaRouter;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(SpaRouter::new("/", "ui").index_file("index.html"));

    axum::Server::bind(&"0.0.0.0:5080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}