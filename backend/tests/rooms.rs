mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

fn room(number: &str) -> serde_json::Value {
    serde_json::json!({ "room_number": number, "name": format!("Stue {number}"), "floor": 2 })
}

#[tokio::test]
async fn admin_creates_and_staff_reads() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;

    let res = app
        .post("/api/v1/rooms", Some(&admin), Some(room("12")))
        .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let created = json(res).await;
    assert_eq!(created["room_number"], "12");

    let list = json(app.get("/api/v1/rooms", Some(&staff)).await).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    let one = app
        .get(
            &format!("/api/v1/rooms/{}", created["id"].as_str().unwrap()),
            Some(&staff),
        )
        .await;
    assert_eq!(one.status(), StatusCode::OK);
}

#[tokio::test]
async fn staff_cannot_manage_rooms() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;
    let res = app
        .post("/api/v1/rooms", Some(&staff), Some(room("12")))
        .await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn clients_cannot_list_rooms() {
    let app = TestApp::spawn().await;
    let (_, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    assert_eq!(
        app.get("/api/v1/rooms", Some(&client)).await.status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn duplicate_room_number_is_a_conflict() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    assert_eq!(
        app.post("/api/v1/rooms", Some(&admin), Some(room("12")))
            .await
            .status(),
        StatusCode::CREATED
    );
    let dup = app
        .post("/api/v1/rooms", Some(&admin), Some(room("12")))
        .await;
    assert_eq!(dup.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn invalid_room_number_is_rejected() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let res = app
        .post(
            "/api/v1/rooms",
            Some(&admin),
            Some(serde_json::json!({ "room_number": "12 A" })),
        )
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_and_filters() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    app.post("/api/v1/rooms", Some(&admin), Some(room("12")))
        .await;
    app.post(
        "/api/v1/rooms",
        Some(&admin),
        Some(serde_json::json!({ "room_number": "31", "name": "Stue 31", "floor": 3 })),
    )
    .await;

    let by_q = json(app.get("/api/v1/rooms?q=31", Some(&admin)).await).await;
    assert_eq!(by_q.as_array().unwrap().len(), 1);

    let by_floor = json(app.get("/api/v1/rooms?floor=2", Some(&admin)).await).await;
    assert_eq!(by_floor[0]["room_number"], "12");
}

#[tokio::test]
async fn update_and_deactivate_are_audited() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let id = json(
        app.post("/api/v1/rooms", Some(&admin), Some(room("12")))
            .await,
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let upd = app
        .patch(
            &format!("/api/v1/rooms/{id}"),
            Some(&admin),
            serde_json::json!({ "name": "Stue 12, Kardiologi" }),
        )
        .await;
    assert_eq!(upd.status(), StatusCode::OK);
    assert_eq!(json(upd).await["name"], "Stue 12, Kardiologi");

    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/rooms/{id}/deactivate"),
            Some(&admin),
            None
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    let inactive = json(app.get("/api/v1/rooms?active=false", Some(&admin)).await).await;
    assert_eq!(inactive.as_array().unwrap().len(), 1);

    let history = json(
        app.get(&format!("/api/v1/audit-log/room/{id}"), Some(&admin))
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
        vec!["room.created", "room.updated", "room.deactivated"]
    );
}

#[tokio::test]
async fn only_admin_can_hard_delete_rooms() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;
    let id = json(
        app.post("/api/v1/rooms", Some(&admin), Some(room("12")))
            .await,
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    assert_eq!(
        app.delete(&format!("/api/v1/rooms/{id}"), Some(&staff))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.delete(&format!("/api/v1/rooms/{id}"), Some(&admin))
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        app.get(&format!("/api/v1/rooms/{id}"), Some(&admin))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    let history = json(
        app.get(&format!("/api/v1/audit-log/room/{id}"), Some(&admin))
            .await,
    )
    .await;
    let last = history.as_array().unwrap().last().unwrap();
    assert_eq!(last["action"], "room.deleted");
    assert_eq!(last["details"]["room_number"], "12");
}
