mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

fn reading(t: f64, h: f64) -> serde_json::Value {
    serde_json::json!({ "temperature_c": t, "humidity_pct": h })
}

#[tokio::test]
async fn breach_raises_once_and_return_to_range_resolves() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let post = |t, h| app.post_as_device("/api/v1/devices/measurements", &dev, &key, reading(t, h));

    post(22.0, 45.0).await;
    let none = json(app.get("/api/v1/alarms?open=true", Some(&admin)).await).await;
    assert_eq!(none.as_array().unwrap().len(), 0);

    post(29.4, 45.0).await; // above the default 26.0
    post(30.1, 45.0).await; // still above: no second alarm
    let open = json(app.get("/api/v1/alarms?open=true", Some(&admin)).await).await;
    assert_eq!(open.as_array().unwrap().len(), 1);
    assert_eq!(open[0]["kind"], "temperature_high");
    assert_eq!(
        open[0]["measured_value"], 29.4,
        "value from the reading that raised it"
    );
    assert_eq!(open[0]["threshold_value"], 26.0);

    post(24.0, 45.0).await; // back in range
    let after = json(app.get("/api/v1/alarms?open=true", Some(&admin)).await).await;
    assert_eq!(after.as_array().unwrap().len(), 0);
    let all = json(
        app.get(&format!("/api/v1/alarms?room_id={room}"), Some(&admin))
            .await,
    )
    .await;
    assert!(all[0]["resolved_at"].is_string());
}

#[tokio::test]
async fn room_override_changes_what_counts_as_a_breach() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    app.put(&format!("/api/v1/rooms/{room}/thresholds"), Some(&admin),
            serde_json::json!({ "temperature_min": 21.0, "temperature_max": 23.0, "humidity_min": 30.0, "humidity_max": 60.0 })).await;

    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(24.0, 45.0),
    )
    .await;
    let open = json(app.get("/api/v1/alarms?open=true", Some(&admin)).await).await;
    assert_eq!(open.as_array().unwrap().len(), 1);
    assert_eq!(open[0]["threshold_value"], 23.0);
}

#[tokio::test]
async fn two_kinds_can_be_open_at_once() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(30.0, 80.0),
    )
    .await;

    let open = json(app.get("/api/v1/alarms?open=true", Some(&admin)).await).await;
    let mut kinds: Vec<_> = open
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["kind"].as_str().unwrap())
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["humidity_high", "temperature_high"]);
}

#[tokio::test]
async fn staff_acknowledges_and_resolves_with_audit() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (staff_id, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(30.0, 45.0),
    )
    .await;
    let id = json(app.get("/api/v1/alarms?open=true", Some(&staff)).await).await[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let ack = app
        .post::<()>(
            &format!("/api/v1/alarms/{id}/acknowledge"),
            Some(&staff),
            None,
        )
        .await;
    assert_eq!(ack.status(), StatusCode::OK);
    assert_eq!(json(ack).await["acknowledged_by"], staff_id.to_string());
    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/alarms/{id}/acknowledge"),
            Some(&staff),
            None
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );

    let resolved = app
        .post::<()>(&format!("/api/v1/alarms/{id}/resolve"), Some(&staff), None)
        .await;
    assert_eq!(resolved.status(), StatusCode::OK);
    assert!(json(resolved).await["resolved_at"].is_string());
    assert_eq!(
        app.post::<()>(&format!("/api/v1/alarms/{id}/resolve"), Some(&staff), None)
            .await
            .status(),
        StatusCode::CONFLICT
    );

    let history = json(
        app.get(&format!("/api/v1/audit-log/alarm/{id}"), Some(&admin))
            .await,
    )
    .await;
    let actions: Vec<_> = history
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        vec!["alarm.acknowledged", "alarm.resolved"],
        "raise is not audited"
    );
}

#[tokio::test]
async fn clients_cannot_see_alarms() {
    let app = TestApp::spawn().await;
    let (_, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    assert_eq!(
        app.get("/api/v1/alarms", Some(&client)).await.status(),
        StatusCode::FORBIDDEN
    );
}
