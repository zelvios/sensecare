mod common;

use axum::http::StatusCode;
use common::{TestApp, json};
use uuid::Uuid;

fn check_in(room: Uuid, user: Uuid) -> serde_json::Value {
    serde_json::json!({ "room_id": room, "user_id": user })
}

#[tokio::test]
async fn staff_checks_client_in_and_out() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("jacob", "jacob-pass-1", "staff").await;
    let (patient, patient_token) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;

    let res = app
        .post("/api/v1/stays", Some(&staff), Some(check_in(room, patient)))
        .await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let stay = json(res).await;
    assert_eq!(stay["room_number"], "12");
    assert!(stay["checked_out_at"].is_null());

    // the client sees their own stay
    let me = json(app.get("/api/v1/stays/me", Some(&patient_token)).await).await;
    assert_eq!(me["id"], stay["id"]);

    let id = stay["id"].as_str().unwrap();
    let out = app
        .post::<()>(&format!("/api/v1/stays/{id}/check-out"), Some(&staff), None)
        .await;
    assert_eq!(out.status(), StatusCode::OK);
    assert!(json(out).await["checked_out_at"].is_string());

    assert_eq!(
        app.get("/api/v1/stays/me", Some(&patient_token))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    let again = app
        .post::<()>(&format!("/api/v1/stays/{id}/check-out"), Some(&staff), None)
        .await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn one_open_stay_per_room_and_per_user() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (p1, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let (p2, _) = app
        .create_user("patient2", "patient-pass-2", "client")
        .await;
    let r1 = app.create_room("12").await;
    let r2 = app.create_room("13").await;

    assert_eq!(
        app.post("/api/v1/stays", Some(&admin), Some(check_in(r1, p1)))
            .await
            .status(),
        StatusCode::CREATED
    );

    let occupied = app
        .post("/api/v1/stays", Some(&admin), Some(check_in(r1, p2)))
        .await;
    assert_eq!(occupied.status(), StatusCode::CONFLICT);
    assert_eq!(json(occupied).await["message"], "room is already occupied");

    let double = app
        .post("/api/v1/stays", Some(&admin), Some(check_in(r2, p1)))
        .await;
    assert_eq!(double.status(), StatusCode::CONFLICT);
    assert_eq!(
        json(double).await["message"],
        "user already has an open stay"
    );
}

#[tokio::test]
async fn only_active_clients_in_active_rooms() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (staff_id, _) = app.create_user("jacob", "jacob-pass-1", "staff").await;
    let (patient, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;

    let not_client = app
        .post(
            "/api/v1/stays",
            Some(&admin),
            Some(check_in(room, staff_id)),
        )
        .await;
    assert_eq!(not_client.status(), StatusCode::BAD_REQUEST);

    app.post::<()>(
        &format!("/api/v1/rooms/{room}/deactivate"),
        Some(&admin),
        None,
    )
    .await;
    let inactive_room = app
        .post("/api/v1/stays", Some(&admin), Some(check_in(room, patient)))
        .await;
    assert_eq!(inactive_room.status(), StatusCode::BAD_REQUEST);

    let missing = app
        .post(
            "/api/v1/stays",
            Some(&admin),
            Some(check_in(Uuid::new_v4(), patient)),
        )
        .await;
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn clients_cannot_check_anyone_in_or_list() {
    let app = TestApp::spawn().await;
    let (patient, token) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;

    assert_eq!(
        app.post("/api/v1/stays", Some(&token), Some(check_in(room, patient)))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.get("/api/v1/stays", Some(&token)).await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.get("/api/v1/stays/me", Some(&token)).await.status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn list_filters_and_audit() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (patient, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;

    let id = json(
        app.post("/api/v1/stays", Some(&admin), Some(check_in(room, patient)))
            .await,
    )
    .await["id"]
        .as_str()
        .unwrap()
        .to_owned();
    app.post::<()>(&format!("/api/v1/stays/{id}/check-out"), Some(&admin), None)
        .await;

    let open = json(app.get("/api/v1/stays?open=true", Some(&admin)).await).await;
    assert_eq!(open.as_array().unwrap().len(), 0);
    let closed = json(
        app.get(
            &format!("/api/v1/stays?open=false&room_id={room}"),
            Some(&admin),
        )
        .await,
    )
    .await;
    assert_eq!(closed.as_array().unwrap().len(), 1);

    let history = json(
        app.get(&format!("/api/v1/audit-log/stay/{id}"), Some(&admin))
            .await,
    )
    .await;
    let actions: Vec<_> = history
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, vec!["stay.checked_in", "stay.checked_out"]);
}
