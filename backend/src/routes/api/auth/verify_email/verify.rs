use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use axum::http::{HeaderMap, StatusCode};
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::{
            CreatableModel, UpdatableModel, user_activity::UserActivity,
            user_email_verification::UserEmailVerification, user_session::UserSession,
        },
        response::{ApiResponse, ApiResponseResult},
    };
    use tower_cookies::Cookies;
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(length(chars, min = 96, max = 96))]
        #[schema(min_length = 96, max_length = 96)]
        token: String,
    }

    #[derive(ToSchema, Serialize)]
    struct Response {
        user: shared::models::user::ApiFullUser,
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        ip: shared::GetIp,
        headers: HeaderMap,
        cookies: Cookies,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let ratelimit = state.settings.get_as(|s| s.ratelimits.auth_login).await?;
        state
            .cache
            .ratelimit(
                "auth/verify-email/verify",
                ratelimit.hits,
                ratelimit.window_seconds,
                ip.to_string(),
            )
            .await?;

        let mut verification =
            match UserEmailVerification::delete_by_token(&state.database, &data.token).await? {
                Some(verification) => verification,
                None => {
                    return ApiResponse::error("invalid or expired token")
                        .with_status(StatusCode::BAD_REQUEST)
                        .ok();
                }
            };

        if !verification.user.verified {
            verification
                .user
                .update(
                    &state,
                    shared::models::user::UpdateUserOptions {
                        verified: Some(true),
                        ..Default::default()
                    },
                )
                .await?;
        }

        if let Err(err) = UserActivity::create(
            &state,
            shared::models::user_activity::CreateUserActivityOptions {
                user_uuid: verification.user.uuid,
                impersonator_uuid: None,
                api_key_uuid: None,
                event: "auth:verify-email".into(),
                ip: Some(ip.0.into()),
                data: serde_json::json!({
                    "user_agent": headers
                        .get("User-Agent")
                        .map(|ua| shared::utils::slice_up_to(ua.to_str().unwrap_or("unknown"), 255))
                        .unwrap_or("unknown"),
                }),
                created: None,
            },
        )
        .await
        {
            tracing::warn!(
                user = %verification.user.uuid,
                "failed to log user activity: {:#?}",
                err
            );
        }

        let key = UserSession::create(
            &state,
            shared::models::user_session::CreateUserSessionOptions {
                user_uuid: verification.user.uuid,
                ip: ip.0.into(),
                user_agent: headers
                    .get("User-Agent")
                    .map(|ua| shared::utils::slice_up_to(ua.to_str().unwrap_or("unknown"), 255))
                    .unwrap_or("unknown")
                    .into(),
            },
        )
        .await?;

        cookies.add(UserSession::get_cookie(&state, key).await?);

        ApiResponse::new_serialized(Response {
            user: verification
                .user
                .into_api_full_object(&state, &state.storage.retrieve_urls().await?)
                .await?,
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
