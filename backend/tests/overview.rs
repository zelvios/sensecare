mod common;

use axum::http::StatusCode;
use common::{TestApp, json};
#[tokio::test]
async fn overview_groups_everything_per_room() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let (patient, _) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    let r1 = app.create_room("12").await;
    let r2 = app.create_room("13").await;
    let (dev, key) = app.create_device(Some(r1)).await;
    app.post(
        "/api/v1/stays",
        Some(&admin),
        Some(serde_json::json!({ "room_id": r1, "user_id": patient })),
    )
    .await;
    app.post_as_device(
        "/api/v1/devices/measurements",
        &dev,
        &key,
        serde_json::json!({ "temperature_c": 30.0, "humidity_pct": 40.0 }),
    )
    .await;
    app.press_button(&dev, &key).await;

    let rows = json(app.get("/api/v1/rooms/overview", Some(&admin)).await).await;
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 2);

    let room12 = rows
        .iter()
        .find(|r| r["room"]["room_number"] == "12")
        .unwrap();
    assert_eq!(room12["latest"]["temperature_c"], 30.0);
    assert_eq!(room12["occupant"]["display_name"], "patient1");
    assert_eq!(room12["device"]["is_active"], true);
    assert_eq!(room12["open_calls"].as_array().unwrap().len(), 1);
    assert_eq!(room12["open_calls"][0]["status"], "open");
    assert_eq!(
        room12["open_alarms"].as_array().unwrap().len(),
        1,
        "30 degrees breaks the default limit"
    );
    assert_eq!(room12["open_alarms"][0]["kind"], "temperature_high");
    assert_eq!(room12["open_alarms"][0]["acknowledged"], false);

    let room13 = rows
        .iter()
        .find(|r| r["room"]["room_number"] == "13")
        .unwrap();
    assert!(room13["latest"].is_null());
    assert!(room13["occupant"].is_null());
    assert!(room13["device"].is_null());
    assert!(room13["open_calls"].as_array().unwrap().is_empty());
    assert!(room13["open_alarms"].as_array().unwrap().is_empty());

    let _ = r2;
}

#[tokio::test]
async fn overview_hides_inactive_rooms_unless_asked() {
    let app = TestApp::spawn().await;
    let admin = app.admin_token().await;
    let room = app.create_room("12").await;
    app.post::<()>(
        &format!("/api/v1/rooms/{room}/deactivate"),
        Some(&admin),
        None,
    )
    .await;

    assert_eq!(
        json(app.get("/api/v1/rooms/overview", Some(&admin)).await)
            .await
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        json(
            app.get("/api/v1/rooms/overview?include_inactive=true", Some(&admin))
                .await
        )
        .await
        .as_array()
        .unwrap()
        .len(),
        1
    );
}

#[tokio::test]
async fn clients_cannot_see_the_overview() {
    let app = TestApp::spawn().await;
    let (_, client) = app
        .create_user("patient1", "patient-pass-1", "client")
        .await;
    assert_eq!(
        app.get("/api/v1/rooms/overview", Some(&client))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
}
