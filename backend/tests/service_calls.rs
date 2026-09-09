mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

#[tokio::test]
async fn button_press_creates_one_open_call_per_room() {
    let app = TestApp::spawn().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    let first = app.press_button(&dev, &key).await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let call = json(first).await;
    assert_eq!(call["status"], "open");

    let second = app.press_button(&dev, &key).await;
    assert_eq!(
        second.status(),
        StatusCode::OK,
        "repeat press returns the existing call"
    );
    assert_eq!(json(second).await["id"], call["id"]);
}

#[tokio::test]
async fn unassigned_device_cannot_call() {
    let app = TestApp::spawn().await;
    let (dev, key) = app.create_device(None).await;
    assert_eq!(
        app.press_button(&dev, &key).await.status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn staff_acknowledges_then_closes_with_note() {
    let app = TestApp::spawn().await;
    let (staff_id, staff) = app.create_user("jacob", "staff-pass-142", "staff").await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let id = json(app.press_button(&dev, &key).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let ack = app
        .post::<()>(
            &format!("/api/v1/service-calls/{id}/acknowledge"),
            Some(&staff),
            None,
        )
        .await;
    assert_eq!(ack.status(), StatusCode::OK);
    let ack = json(ack).await;
    assert_eq!(ack["status"], "in_progress");
    assert_eq!(ack["acknowledged_by"], staff_id.to_string());

    let again = app
        .post::<()>(
            &format!("/api/v1/service-calls/{id}/acknowledge"),
            Some(&staff),
            None,
        )
        .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);

    let closed = app
        .post(
            &format!("/api/v1/service-calls/{id}/close"),
            Some(&staff),
            Some(serde_json::json!({ "note": "Bragt vand" })),
        )
        .await;
    assert_eq!(closed.status(), StatusCode::OK);
    let closed = json(closed).await;
    assert_eq!(closed["status"], "closed");
    assert_eq!(closed["note"], "Bragt vand");
    assert_eq!(closed["closed_by"], staff_id.to_string());

    let reclose = app
        .post(
            &format!("/api/v1/service-calls/{id}/close"),
            Some(&staff),
            Some(serde_json::json!({})),
        )
        .await;
    assert_eq!(reclose.status(), StatusCode::CONFLICT);

    // the room is free for a new call now
    assert_eq!(
        app.press_button(&dev, &key).await.status(),
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn close_directly_from_open_keeps_existing_note() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let id = json(app.press_button(&dev, &key).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    app.patch(
        &format!("/api/v1/service-calls/{id}"),
        Some(&admin),
        serde_json::json!({ "note": "Ringede først" }),
    )
    .await;
    let closed = json(
        app.post(
            &format!("/api/v1/service-calls/{id}/close"),
            Some(&admin),
            Some(serde_json::json!({})),
        )
        .await,
    )
    .await;
    assert_eq!(closed["status"], "closed");
    assert!(
        closed["acknowledged_at"].is_null(),
        "closed without acknowledging is allowed"
    );
    assert_eq!(closed["note"], "Ringede først");
}

#[tokio::test]
async fn list_filters_by_status_and_room() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let r1 = app.create_room("12").await;
    let r2 = app.create_room("13").await;
    let (d1, k1) = app.create_device(Some(r1)).await;
    let (d2, k2) = app.create_device(Some(r2)).await;
    let id1 = json(app.press_button(&d1, &k1).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    app.press_button(&d2, &k2).await;
    app.post(
        &format!("/api/v1/service-calls/{id1}/close"),
        Some(&admin),
        Some(serde_json::json!({})),
    )
    .await;

    let open = json(
        app.get("/api/v1/service-calls?status=open", Some(&admin))
            .await,
    )
    .await;
    assert_eq!(open.as_array().unwrap().len(), 1);
    assert_eq!(open[0]["room_number"], "13");

    let room1 = json(
        app.get(&format!("/api/v1/service-calls?room_id={r1}"), Some(&admin))
            .await,
    )
    .await;
    assert_eq!(room1.as_array().unwrap().len(), 1);
    assert_eq!(room1[0]["status"], "closed");
}

#[tokio::test]
async fn clients_cannot_see_or_handle_calls() {
    let app = TestApp::spawn().await;
    let (_, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let id = json(app.press_button(&dev, &key).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    assert_eq!(
        app.get("/api/v1/service-calls", Some(&client))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/service-calls/{id}/acknowledge"),
            Some(&client),
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn lifecycle_is_audited_with_transitions() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let id = json(app.press_button(&dev, &key).await).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    app.post::<()>(
        &format!("/api/v1/service-calls/{id}/acknowledge"),
        Some(&admin),
        None,
    )
    .await;
    app.patch(
        &format!("/api/v1/service-calls/{id}"),
        Some(&admin),
        serde_json::json!({ "note": "På vej" }),
    )
    .await;
    app.post(
        &format!("/api/v1/service-calls/{id}/close"),
        Some(&admin),
        Some(serde_json::json!({ "note": "Klaret" })),
    )
    .await;

    let history = json(
        app.get(
            &format!("/api/v1/audit-log/service_call/{id}"),
            Some(&admin),
        )
        .await,
    )
    .await;
    let entries = history.as_array().unwrap();
    let actions: Vec<_> = entries
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        vec![
            "service_call.created",
            "service_call.acknowledged",
            "service_call.updated",
            "service_call.closed"
        ]
    );
    assert!(
        entries[0]["actor_id"].is_null(),
        "created by the device, no human actor"
    );
    assert_eq!(entries[1]["details"]["status"]["to"], "in_progress");
    assert_eq!(entries[3]["details"]["status"]["from"], "in_progress");
    assert_eq!(entries[3]["details"]["note"], "Klaret");
}
