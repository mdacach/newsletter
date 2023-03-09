use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};

use crate::session_state::TypedSession;
use crate::utils::{opaque_error_500, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<FormData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(opaque_error_500)?.is_none() {
        return Ok(see_other("/login"));
    }

    let new_password = form.new_password.expose_secret();
    let new_password_check = form.new_password_check.expose_secret();
    if new_password != new_password_check {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(see_other("/admin/password"));
    }
    todo!()
}
