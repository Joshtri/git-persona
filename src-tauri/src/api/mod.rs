pub(crate) mod docs;
pub(crate) mod profile;
pub(crate) mod schemas;

use axum::Router;
use utoipa_swagger_ui::SwaggerUi;

use docs::ApiDoc;
use utoipa::OpenApi as _;

/// Spawns a background Axum server serving the Swagger UI.
/// Available at `http://127.0.0.1:9876/docs` while the app is running.
/// Only active in debug builds to avoid exposing an HTTP port in production.
#[cfg(debug_assertions)]
pub(crate) fn serve_docs() {
    tauri::async_runtime::spawn(async {
        let app = Router::new()
            .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:9876").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("api docs: failed to bind port 9876: {e}");
                return;
            }
        };
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("api docs: server error: {e}");
        }
    });
}
