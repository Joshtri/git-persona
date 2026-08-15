use utoipa::OpenApi;

use super::schemas::{
    CreateProfileBody, ErrorResponse, IdentitySchema, ProfileSchema, UpdateProfileBody,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "GitPersona IPC Reference",
        version = "0.9.2",
        description = "Internal API documentation for GitPersona Tauri commands. \
                       These endpoints mirror the IPC surface exposed to the frontend \
                       via `invoke()` — not an HTTP API in production."
    ),
    paths(
        super::profile::list,
        super::profile::create,
        super::profile::update,
        super::profile::delete,
        super::profile::apply,
        super::profile::get_active,
    ),
    components(schemas(
        ProfileSchema,
        IdentitySchema,
        CreateProfileBody,
        UpdateProfileBody,
        ErrorResponse,
    )),
    tags(
        (name = "Profiles", description = "Git identity profiles"),
    )
)]
pub(crate) struct ApiDoc;
