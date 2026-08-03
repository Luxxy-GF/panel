use super::State;
use utoipa_axum::{router::OpenApiRouter, routes};

mod post {
    use axum::http::StatusCode;
    use garde::Validate;
    use serde::{Deserialize, Serialize};
    use shared::{
        ApiError, GetState,
        models::{ByUuid, CreatableModel, UpdatableModel, user::User, user_session::UserSession},
        response::{ApiResponse, ApiResponseResult},
    };
    use tower_cookies::Cookies;
    use utoipa::ToSchema;

    #[derive(ToSchema, Validate, Deserialize)]
    pub struct Payload {
        #[garde(length(chars, min = 3, max = 15), pattern("^[a-zA-Z0-9_]+$"))]
        #[schema(min_length = 3, max_length = 15)]
        #[schema(pattern = "^[a-zA-Z0-9_]+$")]
        username: String,
        #[garde(email)]
        #[schema(format = "email")]
        email: String,
        #[garde(length(chars, min = 1, max = 255))]
        #[schema(min_length = 1, max_length = 255)]
        name_first: String,
        #[garde(length(chars, min = 1, max = 255))]
        #[schema(min_length = 1, max_length = 255)]
        name_last: String,
        #[garde(length(chars, min = 8, max = 512))]
        #[schema(min_length = 8, max_length = 512)]
        password: String,

        #[garde(skip)]
        captcha: Option<String>,
    }

    #[derive(ToSchema, Serialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum Response {
        /// The account was created and the user has been logged in.
        Completed {
            user: Box<shared::models::user::ApiFullUser>,
        },
        /// The account was created but requires email verification before the user can log in.
        VerificationRequired {},
    }

    #[utoipa::path(post, path = "/", responses(
        (status = OK, body = inline(Response)),
        (status = BAD_REQUEST, body = ApiError),
    ), request_body = inline(Payload))]
    pub async fn route(
        state: GetState,
        ip: shared::GetIp,
        headers: axum::http::HeaderMap,
        cookies: Cookies,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let settings = state.settings.get().await?;
        if !settings.app.registration_enabled {
            return ApiResponse::error("registration is disabled")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let ratelimit = settings.ratelimits.auth_register;
        let require_email_verification = settings.app.registration_require_email_verification;
        let mail_configured = !matches!(settings.mail_mode, shared::settings::MailMode::None);
        drop(settings);

        state
            .cache
            .ratelimit(
                "auth/register",
                ratelimit.hits,
                ratelimit.window_seconds,
                ip.to_string(),
            )
            .await?;

        if let Err(error) = state.captcha.verify(ip, data.captcha).await {
            return ApiResponse::error(&error)
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }

        let user = match User::create_automatic_admin(
            &state.database,
            &data.username,
            &data.email,
            &data.name_first,
            &data.name_last,
            &data.password,
        )
        .await
        {
            Ok(user_uuid) => User::by_uuid(&state.database, user_uuid).await?,
            Err(err) if err.is_unique_violation() => {
                return ApiResponse::error("user with username or email already exists")
                    .with_status(StatusCode::BAD_REQUEST)
                    .ok();
            }
            Err(err) => {
                tracing::error!("failed to create user: {:?}", err);

                return ApiResponse::error("failed to create user")
                    .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .ok();
            }
        };

        // The first user (always an admin) is created during first-run setup, which logs in via the
        // same endpoint; never gate it on verification or a fresh install could lock itself out.
        // If verification is enabled but no mailer is configured, fail open (log a warning) rather
        // than making registration impossible.
        let mut user = user;
        if require_email_verification && !user.admin {
            if !mail_configured {
                tracing::warn!(
                    user = %user.uuid,
                    "registration_require_email_verification is enabled but no mailer is configured; \
                     auto-verifying the new account"
                );
            } else {
                user.update(
                    &state,
                    shared::models::user::UpdateUserOptions {
                        verified: Some(false),
                        ..Default::default()
                    },
                )
                .await?;

                let state = state.clone();
                tokio::spawn(async move {
                    super::super::verify_email::send_verification_email(&state, &user).await;
                });

                return ApiResponse::new_serialized(Response::VerificationRequired {}).ok();
            }
        }

        let key = UserSession::create(
            &state,
            shared::models::user_session::CreateUserSessionOptions {
                user_uuid: user.uuid,
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

        ApiResponse::new_serialized(Response::Completed {
            user: Box::new(
                user.into_api_full_object(&state, &state.storage.retrieve_urls().await?)
                    .await?,
            ),
        })
        .ok()
    }
}

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .routes(routes!(post::route))
        .with_state(state.clone())
}
