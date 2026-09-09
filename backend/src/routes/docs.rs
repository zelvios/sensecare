//! OpenAPI document: metadata, tags and security schemes.
//! Paths and schemas are collected automatically by `OpenApiRouter`.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SenseCare API",
        description = "Indoor climate monitoring and service calls for hospital rooms.\n\n\
            People: log in via POST /api/v1/auth/login, then paste the returned \
            session_token under Authorize -> bearer.\n\n\
            Devices: paste the id and key from device registration under \
            Authorize -> device_id and device_key.",
        version = env!("CARGO_PKG_VERSION"),
    ),
    tags(
        (name = "health", description = "Liveness and index"),
        (name = "auth", description = "Login, logout, current user"),
        (name = "users", description = "Account management: staff manage clients, admins manage everyone"),
        (name = "rooms", description = "Room management: staff read, admins manage"),
        (name = "devices", description = "Room nodes: registration, keys and device authentication"),
        (name = "stays", description = "Which client occupies which room: check-in and check-out"),
        (name = "measurements", description = "Temperature and humidity: reported by devices, read per room"),
        (name = "service_calls", description = "The room button: raised by devices, handled by staff"),
        (name = "audit", description = "Audit log (admin)"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Registers the security schemes.
///   bearer                    people: `security(("bearer" = []))`
///   device_id + device_key    firmware: `security(("device_id" = [], "device_key" = []))`
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );
        components.add_security_scheme(
            "device_id",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Device-Id"))),
        );
        components.add_security_scheme(
            "device_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Device-Key"))),
        );
    }
}
