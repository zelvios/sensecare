#![allow(dead_code)]
//! Shared setup for integration tests.
//!
//! Every test gets its own database (`sensecare_test_<id>`), migrated from
//! scratch, with a bootstrap admin. Nextest then run tests in parallel without them stepping on
//! each other. The database is dropped when the `TestApp` is dropped.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode, header},
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use sensecare_api::config::{Config, Environment};
use serde::Serialize;
use tower::ServiceExt;
use uuid::Uuid;

pub const ADMIN_PASSWORD: &str = "test-admin-password";

pub struct TestApp {
    pub router: Router,
    pub db_name: String,
    admin_url: String,
}

impl TestApp {
    pub async fn spawn() -> TestApp {
        dotenvy::dotenv().ok();
        // Points at the `postgres` maintenance DB where we create a fresh one from there.
        let admin_url = std::env::var("TEST_ADMIN_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://sensecare:change-me@localhost:5433/postgres".into());

        let db_name = format!("sensecare_test_{}", Uuid::new_v4().simple());
        {
            let mut conn = AsyncPgConnection::establish(&admin_url)
                .await
                .expect("connect to postgres");
            diesel::sql_query(format!("CREATE DATABASE {db_name}"))
                .execute(&mut conn)
                .await
                .expect("create test database");
        }

        let database_url = admin_url
            .rsplit_once('/')
            .map(|(base, _)| format!("{base}/{db_name}"))
            .unwrap();

        let config = Config {
            environment: Environment::Development,
            database_url,
            bind_addr: "127.0.0.1:0".into(),
            db_pool_size: 2,
            web_origin: None,
            session_ttl_hours: 1,
            bootstrap_admin_password: Some(ADMIN_PASSWORD.into()),
            api_docs: false,
        };

        let router = sensecare_api::build_app(config).await.expect("build app");
        TestApp {
            router,
            db_name,
            admin_url,
        }
    }

    // --- request helpers ----

    pub async fn get(&self, path: &str, token: Option<&str>) -> Response<Body> {
        self.send(Request::get(path), token, None::<()>).await
    }

    /// GET authenticated as a device, with `X-Device-Id` and `X-Device-Key`.
    pub async fn get_as_device(&self, path: &str, id: &str, key: &str) -> Response<Body> {
        let request = Request::get(path)
            .header("x-device-id", id)
            .header("x-device-key", key)
            .body(Body::empty())
            .unwrap();
        self.router.clone().oneshot(request).await.unwrap()
    }

    /// POST authenticated as a device, with a JSON body.
    pub async fn post_as_device<B: Serialize>(
        &self,
        path: &str,
        id: &str,
        key: &str,
        body: B,
    ) -> Response<Body> {
        let request = Request::post(path)
            .header("x-device-id", id)
            .header("x-device-key", key)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        self.router.clone().oneshot(request).await.unwrap()
    }

    pub async fn post<B: Serialize>(
        &self,
        path: &str,
        token: Option<&str>,
        body: Option<B>,
    ) -> Response<Body> {
        self.send(Request::post(path), token, body).await
    }

    pub async fn patch<B: Serialize>(
        &self,
        path: &str,
        token: Option<&str>,
        body: B,
    ) -> Response<Body> {
        self.send(Request::patch(path), token, Some(body)).await
    }

    pub async fn put<B: Serialize>(
        &self,
        path: &str,
        token: Option<&str>,
        body: B,
    ) -> Response<Body> {
        self.send(Request::put(path), token, Some(body)).await
    }

    pub async fn delete(&self, path: &str, token: Option<&str>) -> Response<Body> {
        self.send(Request::delete(path), token, None::<()>).await
    }

    async fn send<B: Serialize>(
        &self,
        mut builder: axum::http::request::Builder,
        token: Option<&str>,
        body: Option<B>,
    ) -> Response<Body> {
        if let Some(t) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
        }
        let request = match body {
            Some(b) => builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&b).unwrap()))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        };
        self.router.clone().oneshot(request).await.unwrap()
    }

    // --- setup helpers ----

    pub async fn login(&self, username: &str, password: &str) -> String {
        let res = self
            .post(
                "/api/v1/auth/login",
                None,
                Some(serde_json::json!({ "username": username, "password": password })),
            )
            .await;
        assert_eq!(res.status(), StatusCode::OK, "login as {username} failed");
        json(res).await["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    pub async fn admin_token(&self) -> String {
        self.login("admin", ADMIN_PASSWORD).await
    }

    /// Creates a user as admin and returns (id, token for that user).
    pub async fn create_user(&self, username: &str, password: &str, role: &str) -> (Uuid, String) {
        let admin = self.admin_token().await;
        let res = self
            .post(
                "/api/v1/users",
                Some(&admin),
                Some(serde_json::json!({
                    "username": username, "display_name": username, "password": password, "role": role
                })),
            )
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let id: Uuid = json(res).await["id"].as_str().unwrap().parse().unwrap();
        let token = self.login(username, password).await;
        (id, token)
    }

    /// Creates a room as admin and returns its id.
    pub async fn create_room(&self, number: &str) -> Uuid {
        let admin = self.admin_token().await;
        let res = self
            .post(
                "/api/v1/rooms",
                Some(&admin),
                Some(serde_json::json!({ "room_number": number })),
            )
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        json(res).await["id"].as_str().unwrap().parse().unwrap()
    }

    /// Registers a device as admin, optionally assigned to a room. Returns (device id, key).
    pub async fn create_device(&self, room: Option<Uuid>) -> (String, String) {
        let admin = self.admin_token().await;
        let res = self
            .post(
                "/api/v1/devices",
                Some(&admin),
                Some(serde_json::json!({ "label": "node", "room_id": room })),
            )
            .await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let v = json(res).await;
        (
            v["id"].as_str().unwrap().to_owned(),
            v["key"].as_str().unwrap().to_owned(),
        )
    }

    pub async fn press_button(&self, id: &str, key: &str) -> Response<Body> {
        let request = Request::post("/api/v1/devices/service-calls")
            .header("x-device-id", id)
            .header("x-device-key", key)
            .body(Body::empty())
            .unwrap();
        self.router.clone().oneshot(request).await.unwrap()
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // Runs sync in Drop and spawn a task on the runtime to clean up.
        let admin_url = self.admin_url.clone();
        let db_name = self.db_name.clone();
        tokio::spawn(async move {
            if let Ok(mut conn) = AsyncPgConnection::establish(&admin_url).await {
                let _ =
                    diesel::sql_query(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"))
                        .execute(&mut conn)
                        .await;
            }
        });
    }
}

/// Reads a JSON body.
pub async fn json(res: Response<Body>) -> serde_json::Value {
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}
