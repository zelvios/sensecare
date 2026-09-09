//! Acceptance tests, one per requirement the API can prove on its own,
//! named after the K-id in the "krav- og accepttest specifikation".
//!
//! Not covered here and verified with real hardware or the real server:
//!   K8  interval: device reports every <= 120s (firmware, #20)
//!   K9  latency: created_at - pressed_at under 7s (firmware, #20)
//!   K12 redirect: HTTP to HTTPS (reverse proxy, #26)
//!   K13 one-command start and data after restart (docker compose, #26)
//! Frontend display for K3, K4 and K5 is verified by demo (#22 to #25).

mod common;

use axum::http::StatusCode;
use common::{TestApp, json};

fn reading(t: f64, h: f64) -> serde_json::Value {
    serde_json::json!({ "temperature_c": t, "humidity_pct": h })
}

/// K1 / K8: a reading from a device is stored with the correct room, value and timestamp.
#[tokio::test]
async fn k1_k8_measurement_stored_for_correct_room() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    let res = app
        .post_as_device(
            "/api/v1/devices/measurements",
            &dev,
            &key,
            reading(21.4, 43.0),
        )
        .await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let stored = json(
        app.get(
            &format!("/api/v1/rooms/{room}/measurements/latest"),
            Some(&admin),
        )
        .await,
    )
    .await;
    assert_eq!(stored["room_id"], room.to_string());
    assert_eq!(stored["temperature_c"], 21.4);
    assert_eq!(stored["humidity_pct"], 43.0);
    assert!(stored["measured_at"].is_string());
    assert!(stored["received_at"].is_string());
}

/// K2 / K9: a press is registered on the correct room with both timestamps,
/// shows up for staff, and can be marked handled.
#[tokio::test]
async fn k2_k9_button_press_becomes_a_call_staff_can_handle() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;

    let pressed_at = chrono::Utc::now().to_rfc3339();
    let call = json(app.press_button_at(&dev, &key, &pressed_at).await).await;
    assert_eq!(call["room_id"], room.to_string());
    assert!(call["created_at"].is_string());
    assert!(
        call["pressed_at"].is_string(),
        "K9 latency is created_at - pressed_at"
    );

    let open = json(
        app.get("/api/v1/service-calls?status=open", Some(&staff))
            .await,
    )
    .await;
    assert_eq!(open[0]["room_number"], "12");

    let id = call["id"].as_str().unwrap();
    let closed = json(
        app.post(
            &format!("/api/v1/service-calls/{id}/close"),
            Some(&staff),
            Some(serde_json::json!({})),
        )
        .await,
    )
    .await;
    assert_eq!(closed["status"], "closed");
}

/// K3: the system distinguishes the roles at login.
#[tokio::test]
async fn k3_login_reports_the_users_role() {
    let app = TestApp::spawn().await;
    let (_, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;

    assert_eq!(
        json(app.get("/api/v1/auth/me", Some(&client)).await).await["role"],
        "client"
    );
    assert_eq!(
        json(app.get("/api/v1/auth/me", Some(&staff)).await).await["role"],
        "staff"
    );
}

/// K4: a client can read their own room's readings and their check-in time.
#[tokio::test]
async fn k4_client_sees_own_room_and_stay() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (patient, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    app.post(
        "/api/v1/stays",
        Some(&admin),
        Some(serde_json::json!({ "room_id": room, "user_id": patient })),
    )
    .await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(22.0, 40.0),
    )
    .await;

    let stay = json(app.get("/api/v1/stays/me", Some(&client)).await).await;
    assert_eq!(stay["room_number"], "12");
    assert!(stay["checked_in_at"].is_string());
    assert!(stay["checked_out_at"].is_null());

    let history = json(
        app.get(&format!("/api/v1/rooms/{room}/measurements"), Some(&client))
            .await,
    )
    .await;
    assert_eq!(history.as_array().unwrap().len(), 1);
}

/// K5: staff see every registered room and, per room, its readings and number of calls.
#[tokio::test]
async fn k5_staff_overview_lists_all_rooms_with_details() {
    let app = TestApp::spawn().await;
    let (_, staff) = app.create_user("hansen", "nurse-pass-12", "staff").await;
    let r1 = app.create_room("12").await;
    app.create_room("13").await;
    let (dev, key) = app.create_device(Some(r1)).await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(22.0, 40.0),
    )
    .await;
    app.press_button(&dev, &key).await;

    let rooms = json(app.get("/api/v1/rooms", Some(&staff)).await).await;
    let numbers: Vec<_> = rooms
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["room_number"].as_str().unwrap())
        .collect();
    assert_eq!(numbers, vec!["12", "13"]);

    let readings = json(
        app.get(&format!("/api/v1/rooms/{r1}/measurements"), Some(&staff))
            .await,
    )
    .await;
    assert_eq!(readings.as_array().unwrap().len(), 1);

    let calls = json(
        app.get(&format!("/api/v1/service-calls?room_id={r1}"), Some(&staff))
            .await,
    )
    .await;
    assert_eq!(calls.as_array().unwrap().len(), 1);
}

/// K6: readings for a chosen period match what is stored.
#[tokio::test]
async fn k6_history_for_a_chosen_period() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    let (dev, key) = app.create_device(Some(room)).await;
    let january = serde_json::json!({
        "temperature_c": 19.5, "humidity_pct": 40.0, "measured_at": "2026-01-15T10:00:00Z"
    });
    app.post_as_device("/api/v1/devices/measurements", &dev, &key, january)
        .await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        reading(22.0, 40.0),
    )
    .await;

    let period = json(
        app.get(
            &format!(
                "/api/v1/rooms/{room}/measurements?from=2026-01-01T00:00:00Z&to=2026-01-31T23:59:59Z"
            ),
            Some(&admin),
        )
            .await,
    )
        .await;
    assert_eq!(period.as_array().unwrap().len(), 1);
    assert_eq!(period[0]["temperature_c"], 19.5);
}

/// K7: staff/admin create, edit and deactivate rooms and accounts, including username
/// and password. A deactivated account cannot log in.
#[tokio::test]
async fn k7_manage_rooms_and_accounts() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;

    let room = app.create_room("12").await;
    assert_eq!(
        app.patch(
            &format!("/api/v1/rooms/{room}"),
            Some(&admin),
            serde_json::json!({ "name": "Stue 12" })
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/rooms/{room}/deactivate"),
            Some(&admin),
            None
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );

    let (id, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    assert_eq!(
        app.patch(
            &format!("/api/v1/users/{id}"),
            Some(&admin),
            serde_json::json!({ "username": "jens.jensen", "display_name": "Jens Jensen" })
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.post(
            &format!("/api/v1/users/{id}/password"),
            Some(&admin),
            Some(serde_json::json!({ "password": "new-pass-12345" }))
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );

    // new username and password work
    let ok = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "jens.jensen", "password": "new-pass-12345" })),
        )
        .await;
    assert_eq!(ok.status(), StatusCode::OK);

    assert_eq!(
        app.post::<()>(
            &format!("/api/v1/users/{id}/deactivate"),
            Some(&admin),
            None
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );

    // deactivated: correct credentials are refused
    let refused = app
        .post(
            "/api/v1/auth/login",
            None,
            Some(serde_json::json!({ "username": "jens.jensen", "password": "new-pass-12345" })),
        )
        .await;
    assert_eq!(refused.status(), StatusCode::FORBIDDEN);
}

/// K10: readings from two devices land in their own rooms, and a wrong key is rejected.
#[tokio::test]
async fn k10_devices_are_identified_and_authenticated() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let r1 = app.create_room("12").await;
    let r2 = app.create_room("13").await;
    let (d1, k1) = app.create_device(Some(r1)).await;
    let (d2, k2) = app.create_device(Some(r2)).await;

    app.post_as_device(
        "/api/v1/devices/measurements",
        &d1,
        &k1,
        reading(20.0, 40.0),
    )
    .await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &d2,
        &k2,
        reading(25.0, 50.0),
    )
    .await;

    let latest_r1 = json(
        app.get(
            &format!("/api/v1/rooms/{r1}/measurements/latest"),
            Some(&admin),
        )
        .await,
    )
    .await;
    let latest_r2 = json(
        app.get(
            &format!("/api/v1/rooms/{r2}/measurements/latest"),
            Some(&admin),
        )
        .await,
    )
    .await;
    assert_eq!(latest_r1["temperature_c"], 20.0);
    assert_eq!(latest_r2["temperature_c"], 25.0);

    let forged = app
        .post_as_device("/api/v1/devices/measurements", &d1, &k2, reading(99.0, 1.0))
        .await;
    assert_eq!(forged.status(), StatusCode::UNAUTHORIZED);
}

/// K11: a call without a token is rejected, and a client cannot reach another room.
#[tokio::test]
async fn k11_authentication_and_room_ownership() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (patient, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let own = app.create_room("12").await;
    let other = app.create_room("13").await;
    app.post(
        "/api/v1/stays",
        Some(&admin),
        Some(serde_json::json!({ "room_id": own, "user_id": patient })),
    )
    .await;

    assert_eq!(
        app.get("/api/v1/rooms", None).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        app.get(&format!("/api/v1/rooms/{own}/measurements"), Some(&client))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.get(
            &format!("/api/v1/rooms/{other}/measurements"),
            Some(&client)
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
}

/// K12 (storage half): the database holds Argon2 hashes, never the password.
#[tokio::test]
async fn k12_passwords_are_stored_hashed() {
    let app = TestApp::spawn().await;
    app.create_user("patient1", "patient-pass-1", "client")
        .await;

    let hash = app.password_hash_of("patient1").await;
    assert!(hash.starts_with("$argon2id$"));
    assert!(!hash.contains("patient-pass-1"));
}
