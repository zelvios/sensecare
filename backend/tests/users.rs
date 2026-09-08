mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

#[tokio::test]
async fn staff_can_manage_clients_but_not_staff() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "hansen-pass-1", "staff").await;

    let ok = app
        .post("/api/v1/users", Some(&staff), Some(serde_json::json!({
            "username": "patient1", "display_name": "Jens", "password": "patient-pass-1", "role": "client"
        })))
        .await;
    assert_eq!(ok.status(), StatusCode::CREATED);

    let forbidden = app
        .post("/api/v1/users", Some(&staff), Some(serde_json::json!({
            "username": "nielsen", "display_name": "Nielsen", "password": "nielsen-pass-1", "role": "staff"
        })))
        .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn staff_list_only_shows_clients() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "hansen-pass-1", "staff").await;
    app.create_user("patient1", "patient-pass-1", "client")
        .await;

    let res = app.get("/api/v1/users", Some(&staff)).await;
    assert_eq!(res.status(), StatusCode::OK);
    let list = json(res).await;
    let roles: Vec<_> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["role"].as_str().unwrap())
        .collect();
    assert_eq!(roles, vec!["client"]);
}

#[tokio::test]
async fn clients_cannot_touch_accounts() {
    let app = TestApp::spawn().await;
    let (id, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;

    assert_eq!(
        app.get("/api/v1/users", Some(&client)).await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/users/{id}/deactivate"),
            Some(&client),
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn admin_cannot_deactivate_self() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let me = json(app.get("/api/v1/auth/me", Some(&admin)).await).await;
    let id = me["id"].as_str().unwrap();

    let res = app
        .post::<()>(
            &format!("/api/v1/users/{id}/deactivate"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn duplicate_username_is_a_conflict() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let body = serde_json::json!({ "username": "patient1", "display_name": "x", "password": "patient-pass-1", "role": "client" });
    assert_eq!(
        app.post("/api/v1/users", Some(&admin), Some(body.clone()))
            .await
            .status(),
        StatusCode::CREATED
    );
    let dup = app.post("/api/v1/users", Some(&admin), Some(body)).await;
    assert_eq!(dup.status(), StatusCode::CONFLICT);
    assert_eq!(json(dup).await["error"], "already_exists");
}

#[tokio::test]
async fn mutations_are_audited() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (id, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    app.patch(
        &format!("/api/v1/users/{id}"),
        Some(&admin),
        serde_json::json!({ "display_name": "Renamed" }),
    )
    .await;
    app.post::<()>(
        &format!("/api/v1/users/{id}/deactivate"),
        Some(&admin),
        None,
    )
    .await;

    let res = app
        .get(&format!("/api/v1/audit-log/user/{id}"), Some(&admin))
        .await;
    let actions: Vec<_> = json(res)
        .await
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        actions,
        vec!["user.created", "user.updated", "user.deactivated"]
    );
}
