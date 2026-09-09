mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

fn limits(tmin: f64, tmax: f64) -> serde_json::Value {
    serde_json::json!({ "temperature_min": tmin, "temperature_max": tmax, "humidity_min": 30.0, "humidity_max": 60.0 })
}

#[tokio::test]
async fn global_default_is_seeded_and_staff_can_read_it() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "hansen-pass-1", "staff").await;
    let res = app.get("/api/v1/thresholds", Some(&staff)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let t = json(res).await;
    assert_eq!(t["source"], "global");
    assert_eq!(t["temperature_max"], 26.0);
}

#[tokio::test]
async fn only_admin_changes_thresholds() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (_, staff) = app.create_user("hansen", "hansen-pass-1", "staff").await;

    assert_eq!(
        app.put("/api/v1/thresholds", Some(&staff), limits(18.0, 25.0))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );

    let ok = app
        .put("/api/v1/thresholds", Some(&admin), limits(18.0, 25.0))
        .await;
    assert_eq!(ok.status(), StatusCode::OK);
    assert_eq!(json(ok).await["temperature_min"], 18.0);

    assert_eq!(
        app.put("/api/v1/thresholds", Some(&admin), limits(25.0, 18.0))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn room_override_and_fallback() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;

    let before = json(
        app.get(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(before["source"], "global");

    let set = app
        .put(
            &format!("/api/v1/rooms/{room}/thresholds"),
            Some(&admin),
            limits(21.0, 24.0),
        )
        .await;
    assert_eq!(set.status(), StatusCode::OK);
    let after = json(
        app.get(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(after["source"], "room");
    assert_eq!(after["temperature_max"], 24.0);

    assert_eq!(
        app.delete(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let reset = json(
        app.get(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(reset["source"], "global");

    assert_eq!(
        app.delete(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    let history = json(
        app.get("/api/v1/audit-log?entity_type=threshold", Some(&admin))
            .await,
    )
    .await;
    let actions: Vec<_> = history
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, vec!["threshold.removed", "threshold.updated"]);
}
