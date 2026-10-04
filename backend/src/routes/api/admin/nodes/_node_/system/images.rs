use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod get {
    use shared::{
        GetState,
        models::{node::GetNode, user::GetPermissionManager},
        response::{ApiResponse, ApiResponseResult},
    };

    #[utoipa::path(get, path = "/", responses((status = OK, body = serde_json::Value)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        node: GetNode,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        let images: serde_json::Value = node
            .api_client(&state.database)
            .await?
            .request_raw(reqwest::Method::GET, "/api/system/images")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        ApiResponse::new_serialized(images).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(get::route))
        .with_state(state.clone())
}
