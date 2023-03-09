use actix_web::{web, HttpResponse};
use secrecy::Secret;

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
    todo!()
}
