mod common;

use axum::http::StatusCode;
use common::{ADMIN_PASSWORD, TestApp, json};

#[tokio::test]
async fn unauthenticated_request_is_rejected() {
    let app = TestApp::spawn().await;
    let res = app.get("/api/v1/auth/me", None).await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json(res).await["error"], "unauthorized");
}

#[tokio::test]
async fn wrong_password_and_unknown_user_look_identical() {
    let app = TestApp::spawn().await;
    let wrong = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "admin", "password": "nope" })),
        )
        .await;
    let unknown = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "ghost", "password": "nope" })),
        )
        .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json(wrong).await, json(unknown).await);
}

#[tokio::test]
async fn login_then_me_then_logout() {
    let app = TestApp::spawn().await;
    let token = app.login("admin", ADMIN_PASSWORD).await;

    let me = app.get("/api/v1/auth/me", Some(&token)).await;
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(json(me).await["role"], "admin");

    let out = app
        .post::<()>("/api/v1/auth/logout", Some(&token), None)
        .await;
    assert_eq!(out.status(), StatusCode::NO_CONTENT);

    let after = app.get("/api/v1/auth/me", Some(&token)).await;
    assert_eq!(after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn deactivated_account_loses_its_session() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (id, client_token) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;

    let res = app
        .post::<()>(
            &format!("/api/v1/users/{id}/deactivate"),
            Some(&admin),
            None,
        )
        .await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // Existing session is gone.
    let me = app.get("/api/v1/auth/me", Some(&client_token)).await;
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);

    // A fresh login is refused with the specific reason (correct password, inactive account).
    let login = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "patient1", "password": "patient-pass-1" })),
        )
        .await;
    assert_eq!(login.status(), StatusCode::FORBIDDEN);
    assert_eq!(json(login).await["error"], "account_deactivated");
}

#[tokio::test]
async fn logins_and_failures_are_audited() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (id, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;

    // wrong password
    let bad = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "patient1", "password": "wrong-pass-1" })),
        )
        .await;
    assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

    // logout from the session created by create_user
    let token = app.login("patient1", "patient-pass-1").await;
    app.post::<()>("/api/v1/auth/logout", Some(&token), None)
        .await;

    let history = json(
        app.get(&format!("/api/v1/audit-log/user/{id}"), Some(&admin))
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
        vec![
            "user.created",
            "user.logged_in",
            "user.login_failed",
            "user.logged_in",
            "user.logged_out"
        ]
    );

    let logged_in = history
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["action"] == "user.logged_in")
        .unwrap();
    assert!(logged_in["details"]["session_id"].is_string());
}

#[tokio::test]
async fn self_service_password_change() {
    let app = TestApp::spawn().await;
    let (_, token) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let other = app.login("patient1", "patient-pass-1").await;

    let wrong = app
        .post(
            "/api/v1/auth/password",
            Some(&token),
            Some(
                serde_json::json!({ "current_password": "nope", "new_password": "new-pass-12345" }),
            ),
        )
        .await;
    assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);

    let ok = app.post("/api/v1/auth/password", Some(&token), Some(serde_json::json!({ "current_password": "patient-pass-1", "new_password": "new-pass-12345" }))).await;
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);

    assert_eq!(
        app.get("/api/v1/auth/me", Some(&token)).await.status(),
        StatusCode::OK,
        "current session kept"
    );
    assert_eq!(
        app.get("/api/v1/auth/me", Some(&other)).await.status(),
        StatusCode::UNAUTHORIZED,
        "other sessions ended"
    );
    assert_eq!(app.login("patient1", "new-pass-12345").await.len(), 36);
}
