use super::State;
use utoipa_axum::router::OpenApiRouter;

mod resend;
mod verify;

/// Creates an email verification token for the user and sends them the verification email.
///
/// This is best-effort: any failure is logged and swallowed, since the caller (registration or a
/// resend request) must not leak whether a given account exists or is already verified.
pub async fn send_verification_email(state: &shared::State, user: &shared::models::user::User) {
    let token = match shared::models::user_email_verification::UserEmailVerification::create(
        &state.database,
        user.uuid,
    )
    .await
    {
        Ok(token) => token,
        Err(err) => {
            tracing::warn!(
                user = %user.uuid,
                "failed to create email verification token: {:#?}",
                err
            );
            return;
        }
    };

    let settings = match state.settings.get().await {
        Ok(settings) => settings,
        Err(err) => {
            tracing::warn!(
                user = %user.uuid,
                "failed to get settings for email verification email: {:#?}",
                err
            );
            return;
        }
    };

    let verify_link = format!(
        "{}/auth/verify-email?token={}",
        settings.app.url,
        urlencoding::encode(&token),
    );
    drop(settings);

    state
        .mail
        .send_template(
            state,
            "email_verification",
            user.email.clone(),
            minijinja::context! {
                user => user,
                verify_link => verify_link,
            },
        )
        .await;
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/verify", verify::router(state))
        .nest("/resend", resend::router(state))
        .with_state(state.clone())
}
