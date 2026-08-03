use super::State;
use utoipa_axum::router::OpenApiRouter;

mod login;
mod oauth;
mod password;
mod register;
mod verify_email;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/login", login::router(state))
        .nest("/register", register::router(state))
        .nest("/password", password::router(state))
        .nest("/oauth", oauth::router(state))
        .nest("/verify-email", verify_email::router(state))
        .with_state(state.clone())
}
