/// Dummy handlers — never called at runtime.
/// Their only purpose is to give utoipa `#[utoipa::path]` metadata.
/// Actual logic lives in `commands/profiles.rs`.
use super::schemas::{CreateProfileBody, ErrorResponse, ProfileSchema, UpdateProfileBody};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/profiles",
    tag = "Profiles",
    responses(
        (status = 200, description = "List all profiles", body = Vec<ProfileSchema>),
        (status = 500, description = "Internal error",    body = ErrorResponse),
    )
)]
pub(crate) async fn list() {}

#[utoipa::path(
    post,
    path = "/profiles",
    tag = "Profiles",
    request_body = CreateProfileBody,
    responses(
        (status = 201, description = "Profile created", body = ProfileSchema),
        (status = 422, description = "Validation error", body = ErrorResponse),
    )
)]
pub(crate) async fn create() {}

#[utoipa::path(
    put,
    path = "/profiles/{id}",
    tag = "Profiles",
    params(("id" = Uuid, Path, description = "Profile UUID")),
    request_body = UpdateProfileBody,
    responses(
        (status = 200, description = "Profile updated", body = ProfileSchema),
        (status = 404, description = "Not found",       body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    )
)]
pub(crate) async fn update() {}

#[utoipa::path(
    delete,
    path = "/profiles/{id}",
    tag = "Profiles",
    params(("id" = Uuid, Path, description = "Profile UUID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found", body = ErrorResponse),
    )
)]
pub(crate) async fn delete() {}

#[utoipa::path(
    post,
    path = "/profiles/{id}/apply",
    tag = "Profiles",
    params(("id" = Uuid, Path, description = "Profile UUID")),
    responses(
        (status = 204, description = "Profile applied as global git identity"),
        (status = 404, description = "Not found", body = ErrorResponse),
    )
)]
pub(crate) async fn apply() {}

#[utoipa::path(
    get,
    path = "/profiles/active",
    tag = "Profiles",
    responses(
        (status = 200, description = "Currently active profile or null", body = Option<ProfileSchema>),
    )
)]
pub(crate) async fn get_active() {}
