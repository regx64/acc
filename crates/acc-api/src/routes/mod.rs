pub mod admin;
pub mod auth;
pub mod authoring;
pub mod board;
pub mod me;
pub mod problems;
pub mod site;
pub mod submissions;
pub mod users;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(title = "acc API", version = "1", description = "acc 온라인 저지 REST API. 목록은 커서 페이지네이션, 인증은 HttpOnly 세션 쿠키."),
    servers((url = "/api/v1")),
    components(schemas(crate::error::ErrorBody))
)]
struct ApiDoc;

/// Every `/api/v1` route with its OpenAPI description.
pub fn api() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(site::router())
        .merge(auth::router())
        .merge(me::router())
        .merge(users::router())
        .merge(problems::router())
        .merge(submissions::router())
        .merge(board::router())
        .merge(authoring::router())
        .merge(admin::router())
}
