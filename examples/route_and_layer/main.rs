use axum::{extract::Extension, handler::get};
use axum_router::RouterBuilder;
use http::uri::Uri;

mod middleware;

use middleware::{NotFoundLayer, TestMiddlewareLayer};

#[tokio::main]
async fn main() {
    let app = RouterBuilder::new()
        .layer(NotFoundLayer::new())
        .route("/", get(handler))
        .route("/a", get(handler))
        .route("/b", get(handler))
        .route("/c", get(handler))
        .route("/d", get(handler))
        .layer(TestMiddlewareLayer::new("first middleware"))
        .route("/e", get(handler))
        .route("/f", get(handler))
        .route("/g", get(handler))
        .route("/h", get(handler))
        .route("/i", get(handler))
        .layer(TestMiddlewareLayer::new("second middleware"))
        .route("/j", get(handler))
        .route("/k", get(handler))
        .route("/l", get(handler))
        .route("/m", get(handler))
        .route("/n", get(handler))
        .layer(TestMiddlewareLayer::new("third middleware"))
        .route("/o", get(handler))
        .route("/p", get(handler))
        .route("/q", get(handler))
        .route("/r", get(handler))
        .route("/s", get(handler))
        .layer(TestMiddlewareLayer::new("fourth middleware"))
        .route("/t", get(handler))
        .route("/u", get(handler))
        .route("/v", get(handler))
        .route("/w", get(handler))
        .route("/x", get(handler))
        .route("/y", get(handler))
        .route("/z", get(handler))
        .build();

    axum_server::bind("127.0.0.1:3000")
        .serve(app)
        .await
        .unwrap();
}

async fn handler(uri: Uri, vec: Option<Extension<Vec<String>>>) -> String {
    format!("Path: {}\n{:?}", uri.path(), vec)
}
