use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Shared error envelope returned by all endpoints on failure.
#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ErrorResponse {
    /// Machine-readable error code (e.g. `NOT_FOUND`, `VALIDATION`).
    pub(crate) code: String,
    /// Human-readable description.
    pub(crate) message: String,
}

// ---------------------------------------------------------------------------
// Profile schemas
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct IdentitySchema {
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) signing_key: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ProfileSchema {
    pub(crate) id: Uuid,
    pub(crate) label: String,
    pub(crate) identity: IdentitySchema,
    pub(crate) color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct CreateProfileBody {
    pub(crate) label: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) signing_key: Option<String>,
    pub(crate) color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub(crate) struct UpdateProfileBody {
    pub(crate) label: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) signing_key: Option<String>,
    pub(crate) color: Option<String>,
}
