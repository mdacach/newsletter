use actix_web::error::InternalError;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::admin::dashboard::get_username;
use crate::session_state::TypedSession;
use crate::utils::{opaque_error_500, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

async fn reject_anonymous_users(session: TypedSession) -> Result<Uuid, actix_web::Error> {
    match session.get_user_id().map_err(opaque_error_500)? {
        Some(user_id) => Ok(user_id),
        None => {
            // Redirect to login if user is not authenticated.
            let response = see_other("/login");
            let error = anyhow::anyhow!("The user has not logged in.");
            Err(InternalError::from_response(error, response).into())
        }
    }
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = reject_anonymous_users(session).await?;

    let new_password = form.new_password.expose_secret();
    let new_password_check = form.new_password_check.expose_secret();
    if new_password != new_password_check {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }

    let username = get_username(user_id, &pool)
        .await
        .map_err(opaque_error_500)?;
    let credentials = Credentials {
        username,
        password: form.0.current_password.clone(),
    };
    if let Err(error) = validate_credentials(credentials, &pool).await {
        return match error {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(opaque_error_500(error)),
        };
    }

    if new_password.len() < 12 || new_password.len() > 128 {
        FlashMessage::error("Password length must be between 12 and 128.").send();
        return Ok(see_other("/admin/password"));
    }

    crate::authentication::change_password(user_id, form.0.new_password, &pool)
        .await
        .map_err(opaque_error_500)?;
    FlashMessage::info("Your password has been changed.").send();

    Ok(see_other("/admin/password"))
}
