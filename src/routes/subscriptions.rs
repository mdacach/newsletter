use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
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
#[tracing::instrument(name = "Adding a new subscriber",
skip(form, pool),
fields(
request_id = % Uuid::new_v4(),
subscriber_email = % form.email,
subscriber_name = % form.name
))]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
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

    insert_query.execute(pool).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
