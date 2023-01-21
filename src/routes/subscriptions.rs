use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

// This tells serde to implement deserialization for us
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// Actix will try to extract the arguments (in this case web::Form) from the
// request with from_request. (Internally it will try to deserialize the body
// into FormData leveraging serde_urlencoded).
// If this fails (for any argument passed, in this case we only have one),
// it returns 400 BAD REQUEST
// otherwise, the arguments are "populated" and the function is invoked
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    log::info!(
        "Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );
    // This only runs when we execute it with some connection
    let insert_query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    );

    // This is a result, as the query may fail
    match insert_query.execute(pool.get_ref()).await {
        Ok(_) => {
            log::info!("New subscriber details have been saved");
            HttpResponse::Ok()
        }
        Err(e) => {
            // We use debug formatting here because we want access to more information
            // about the error in the logs.
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError()
        }
    }
}
