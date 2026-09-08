mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

async fn register(app: &TestApp, admin: &str, room_id: Option<uuid::Uuid>) -> serde_json::Value {
    let res = app
        .post(
            "/api/v1/devices",
            Some(admin),
            Some(serde_json::json!({ "label": "node", "room_id": room_id })),
        )
        .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    json(res).await
}

#[tokio::test]
async fn register_returns_key_once_and_never_again() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let created = register(&app, &admin, None).await;
    assert_eq!(created["key"].as_str().unwrap().len(), 64);

    let fetched = json(
        app.get(
            &format!("/api/v1/devices/{}", created["id"].as_str().unwrap()),
            Some(&admin),
        )
        .await,
    )
    .await;
    assert!(fetched.get("key").is_none());
    assert!(fetched.get("key_hash").is_none());
}

#[tokio::test]
async fn device_authenticates_with_id_and_key() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let created = register(&app, &admin, None).await;
    let (id, key) = (
        created["id"].as_str().unwrap(),
        created["key"].as_str().unwrap(),
    );

    let ok = app.get_as_device("/api/v1/devices/self", id, key).await;
    assert_eq!(ok.status(), StatusCode::OK);
    assert!(
        json(ok).await["last_seen_at"].is_string(),
        "last_seen_at stamped"
    );

    let bad_key = app.get_as_device("/api/v1/devices/self", id, "0000").await;
    assert_eq!(bad_key.status(), StatusCode::UNAUTHORIZED);

    let no_headers = app.get("/api/v1/devices/self", None).await;
    assert_eq!(no_headers.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rotating_the_key_invalidates_the_old_one() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let created = register(&app, &admin, None).await;
    let id = created["id"].as_str().unwrap();
    let old_key = created["key"].as_str().unwrap();

    let rotated = json(
        app.post::<()>(
            &format!("/api/v1/devices/{id}/rotate-key"),
            Some(&admin),
            None,
        )
        .await,
    )
    .await;
    let new_key = rotated["key"].as_str().unwrap();

    assert_eq!(
        app.get_as_device("/api/v1/devices/self", id, old_key)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.get_as_device("/api/v1/devices/self", id, new_key)
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn deactivated_device_is_rejected() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let created = register(&app, &admin, None).await;
    let (id, key) = (
        created["id"].as_str().unwrap(),
        created["key"].as_str().unwrap(),
    );

    app.post::<()>(
        &format!("/api/v1/devices/{id}/deactivate"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(
        app.get_as_device("/api/v1/devices/self", id, key)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );

    app.post::<()>(
        &format!("/api/v1/devices/{id}/activate"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(
        app.get_as_device("/api/v1/devices/self", id, key)
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn one_active_device_per_room() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;

    register(&app, &admin, Some(room)).await;
    let second = app
        .post(
            "/api/v1/devices",
            Some(&admin),
            Some(serde_json::json!({ "label": "node2", "room_id": room })),
        )
        .await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert_eq!(json(second).await["error"], "conflict");
}

#[tokio::test]
async fn assign_and_unassign_are_audited() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let id = register(&app, &admin, None).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let assigned = app
        .post(
            &format!("/api/v1/devices/{id}/assign"),
            Some(&admin),
            Some(serde_json::json!({ "room_id": room })),
        )
        .await;
    assert_eq!(assigned.status(), StatusCode::OK);
    assert_eq!(json(assigned).await["room_id"], room.to_string());

    let unassigned = app
        .post(
            &format!("/api/v1/devices/{id}/assign"),
            Some(&admin),
            Some(serde_json::json!({ "room_id": null })),
        )
        .await;
    assert!(json(unassigned).await["room_id"].is_null());

    let history = json(
        app.get(&format!("/api/v1/audit-log/device/{id}"), Some(&admin))
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
        vec!["device.registered", "device.assigned", "device.assigned"]
    );
}

#[tokio::test]
async fn staff_can_view_but_not_manage() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (_, staff) = app.create_user("hansen", "hansen-pass-1", "staff").await;
    let id = register(&app, &admin, None).await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    assert_eq!(
        app.get("/api/v1/devices", Some(&staff)).await.status(),
        StatusCode::OK
    );
    assert_eq!(
        app.post("/api/v1/devices", Some(&staff), Some(serde_json::json!({})))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/devices/{id}/rotate-key"),
            Some(&staff),
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.delete(&format!("/api/v1/devices/{id}"), Some(&staff))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn room_with_device_cannot_be_hard_deleted() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    register(&app, &admin, Some(room)).await;

    let res = app
        .delete(&format!("/api/v1/rooms/{room}"), Some(&admin))
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(res).await["error"], "invalid_reference");
}

#[tokio::test]
async fn admin_hard_deletes_unused_device() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let id = register(&app, &admin, None).await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        app.delete(&format!("/api/v1/devices/{id}"), Some(&admin))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        app.get(&format!("/api/v1/devices/{id}"), Some(&admin))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}
