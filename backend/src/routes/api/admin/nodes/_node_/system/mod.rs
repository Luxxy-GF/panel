use super::State;
use utoipa_axum::router::OpenApiRouter;

mod images;
mod ips;
mod logs;
mod overview;
mod runtime;
mod stats;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/ips", ips::router(state))
        .nest("/runtime", runtime::router(state))
        .nest("/images", images::router(state))
        .nest("/logs", logs::router(state))
        .nest("/overview", overview::router(state))
        .nest("/stats", stats::router(state))
        .with_state(state.clone())
}
