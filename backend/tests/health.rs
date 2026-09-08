mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

#[tokio::test]
async fn health_reports_database_connected() {
    let app = TestApp::spawn().await;
    let res = app.get("/health", None).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(json(res).await["database"], "connected");
}
