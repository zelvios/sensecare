//! OpenAPI document: metadata, tags and security scheme.
//! Paths and schemas are collected automatically by `OpenApiRouter`.

use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SenseCare API",
        description = "Indoor climate monitoring and service calls for hospital rooms.\n\n\
            Authentication: log in via POST /api/v1/auth/login, then paste the returned \
            session_token under Authorize -> bearer. Every other endpoint requires it.",
        version = env!("CARGO_PKG_VERSION"),
    ),
    tags(
        (name = "health", description = "Liveness and index"),
        (name = "auth", description = "Login, logout, current user"),
        (name = "users", description = "Account management: staff manage clients, admins manage everyone"),
        (name = "audit", description = "Audit log (admin)"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Registers the bearer scheme. Protected endpoints reference it with `security(("bearer" = []))`.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
        );
    }
}
