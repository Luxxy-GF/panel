use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use axum::http::StatusCode;
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::user::User,
        response::{ApiResponse, ApiResponseResult},
    };
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(email, length(max = 255))]
        #[schema(format = "email", max_length = 255)]
        email: String,

        #[garde(skip)]
        captcha: Option<String>,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {}

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        ip: shared::GetIp,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ratelimit = state
            .settings
            .get_as(|s| s.ratelimits.auth_password_forgot)
            .await?;
        state
            .cache
            .ratelimit(
                "auth/verify-email/resend",
                ratelimit.hits,
                ratelimit.window_seconds,
                ip.to_string(),
            )
            .await?;
        state
            .cache
            .ratelimit(
                "auth/verify-email/resend:email",
                ratelimit.hits,
                ratelimit.window_seconds,
                &data.email,
            )
            .await?;

        if let Err(error) = state.captcha.verify(ip, data.captcha).await {
            return ApiResponse::error(&error)
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        // Only send if the account exists and is not already verified. Always return the same
        // response either way so this endpoint can't be used to enumerate accounts.
        if let Some(user) = User::by_email(&state.database, &data.email).await?
            && !user.verified
        {
            tokio::spawn(async move {
                super::super::send_verification_email(&state, &user).await;
            });
        }

        ApiResponse::new_serialized(Response {}).ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
