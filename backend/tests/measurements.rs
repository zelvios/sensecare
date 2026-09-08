mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

fn reading(t: f64, h: f64) -> serde_json::Value {
    serde_json::json!({ "temperature_c": t, "humidity_pct": h })
}

#[tokio::test]
async fn device_reports_and_staff_reads_history() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    let res = app
        .post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            &key,
            reading(21.37, 43.0),
        )
        .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let stored = json(res).await;
    assert_eq!(stored["temperature_c"], 21.4, "rounded to one decimal");
    assert_eq!(stored["room_id"], room.to_string());

    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(22.0, 44.0),
    )
    .await;

    let history = json(
        app.get(&format!("/api/v1/rooms/{room}/measurements"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(history.as_array().unwrap().len(), 2);
    assert_eq!(history[0]["temperature_c"], 22.0, "newest first");

    let latest = json(
        app.get(
            &format!("/api/v1/rooms/{room}/measurements/latest"),
            Some(&admin),
        )
        .await,
    )
    .await;
    assert_eq!(latest["temperature_c"], 22.0);
}

#[tokio::test]
async fn unassigned_device_cannot_report() {
    let app = TestApp::spawn().await;
    let (dev, key) = app.create_device(None).await;
    let res = app
        .post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            &key,
            reading(21.0, 40.0),
        )
        .await;
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn invalid_readings_are_rejected() {
    let app = TestApp::spawn().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    assert_eq!(
        app.post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            &key,
            reading(90.0, 40.0)
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        app.post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            &key,
            reading(21.0, 101.0)
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );

    let future = serde_json::json!({ "temperature_c": 21.0, "humidity_pct": 40.0, "measured_at": "2099-01-01T00:00:00Z" });
    assert_eq!(
        app.post_as_device("/api/v1/devices/measurements", &dev, &key, future)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );

    let wrong_key = app
        .post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            "0000",
            reading(21.0, 40.0),
        )
        .await;
    assert_eq!(wrong_key.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn client_reads_only_the_room_they_occupy() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (patient, token) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let own = app.create_room("12").await;
    let other = app.create_room("13").await;
    let (dev, key) = app.create_device(Some(own)).await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(21.0, 40.0),
    )
    .await;

    // not checked in anywhere yet
    assert_eq!(
        app.get(&format!("/api/v1/rooms/{own}/measurements"), Some(&token))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );

    app.post(
        "/api/v1/stays",
        Some(&admin),
        Some(serde_json::json!({ "room_id": own, "user_id": patient })),
    )
    .await;

    assert_eq!(
        app.get(&format!("/api/v1/rooms/{own}/measurements"), Some(&token))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.get(
            &format!("/api/v1/rooms/{own}/measurements/latest"),
            Some(&token)
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.get(&format!("/api/v1/rooms/{other}/measurements"), Some(&token))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn history_respects_time_range() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    let old = serde_json::json!({ "temperature_c": 19.0, "humidity_pct": 40.0, "measured_at": "2026-01-01T10:00:00Z" });
    app.post_as_device("/api/v1/devices/measurements", &dev, &key, old)
        .await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(22.0, 40.0),
    )
    .await;

    let default = json(
        app.get(&format!("/api/v1/rooms/{room}/measurements"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(
        default.as_array().unwrap().len(),
        1,
        "default window is the last 24 h"
    );

    let january = json(
        app.get(&format!("/api/v1/rooms/{room}/measurements?from=2026-01-01T00:00:00Z&to=2026-01-02T00:00:00Z"), Some(&admin)).await,
    )
        .await;
    assert_eq!(january.as_array().unwrap().len(), 1);
    assert_eq!(january[0]["temperature_c"], 19.0);

    let bad = app.get(&format!("/api/v1/rooms/{room}/measurements?from=2026-02-01T00:00:00Z&to=2026-01-01T00:00:00Z"), Some(&admin)).await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rooms_and_devices_with_readings_cannot_be_hard_deleted() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(21.0, 40.0),
    )
    .await;

    let d = app
        .delete(&format!("/api/v1/devices/{dev}"), Some(&admin))
        .await;
    assert_eq!(d.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(d).await["error"], "invalid_reference");
}
