#![recursion_limit = "1024"]

use axum::{Router, handler::get};

mod middleware;

use middleware::NoopLayer;

#[tokio::main]
async fn main() {
    tokio::spawn(no_routing("127.0.0.1:3000"));
    tokio::spawn(light_routing("127.0.0.1:3001"));
    tokio::spawn(heavy_routing("127.0.0.1:3002"));
    tokio::spawn(light_routing_middleware("127.0.0.1:3003"));
    tokio::spawn(heavy_routing_middleware("127.0.0.1:3004"));

    std::future::pending::<()>().await;
}

async fn no_routing(addr: &'static str) {
    let app = get(handler);

    axum_server::bind(addr)
        .serve(app)
        .await
        .unwrap();
}

async fn light_routing(addr: &'static str) {
    let app = Router::new().route("/", get(handler));

    axum_server::bind(addr)
        .serve(app)
        .await
        .unwrap();
}

async fn heavy_routing(addr: &'static str) {
    let app = Router::new()
        .route("/", get(handler))
        .route("/a", get(handler))
        .route("/b", get(handler))
        .route("/c", get(handler))
        .route("/d", get(handler))
        .route("/e", get(handler))
        .route("/f", get(handler))
        .route("/g", get(handler))
        .route("/h", get(handler))
        .route("/i", get(handler))
        .route("/j", get(handler))
        .route("/k", get(handler))
        .route("/l", get(handler))
        .route("/m", get(handler))
        .route("/n", get(handler))
        .route("/o", get(handler))
        .route("/p", get(handler))
        .route("/q", get(handler))
        .route("/r", get(handler))
        .route("/s", get(handler))
        .route("/t", get(handler))
        .route("/u", get(handler))
        .route("/v", get(handler))
        .route("/w", get(handler))
        .route("/x", get(handler))
        .route("/y", get(handler))
        .route("/z", get(handler));

    axum_server::bind(addr)
        .serve(app)
        .await
        .unwrap();
}

async fn light_routing_middleware(addr: &'static str) {
    let app = Router::new()
        .layer(NoopLayer::new())
        .route("/", get(handler))
        .layer(NoopLayer::new())
        .layer(NoopLayer::new())
        .layer(NoopLayer::new())
        .layer(NoopLayer::new());

    axum_server::bind(addr)
        .serve(app)
        .await
        .unwrap();
}

async fn heavy_routing_middleware(addr: &'static str) {
    let app = Router::new()
        .layer(NoopLayer::new())
        .route("/", get(handler))
        .route("/a", get(handler))
        .route("/b", get(handler))
        .route("/c", get(handler))
        .route("/d", get(handler))
        .layer(NoopLayer::new())
        .route("/e", get(handler))
        .route("/f", get(handler))
        .route("/g", get(handler))
        .route("/h", get(handler))
        .route("/i", get(handler))
        .layer(NoopLayer::new())
        .route("/j", get(handler))
        .route("/k", get(handler))
        .route("/l", get(handler))
        .route("/m", get(handler))
        .route("/n", get(handler))
        .layer(NoopLayer::new())
        .route("/o", get(handler))
        .route("/p", get(handler))
        .route("/q", get(handler))
        .route("/r", get(handler))
        .route("/s", get(handler))
        .layer(NoopLayer::new())
        .route("/t", get(handler))
        .route("/u", get(handler))
        .route("/v", get(handler))
        .route("/w", get(handler))
        .route("/x", get(handler))
        .route("/y", get(handler))
        .route("/z", get(handler));

    axum_server::bind(addr)
        .serve(app)
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, world!"
}
