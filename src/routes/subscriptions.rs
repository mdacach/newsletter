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
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    // Spans are more powerful, have related metadata and can be entered/exited multiple times
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id, // % means to use Display
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    // Using `enter` in an async function is not great because it can be polled
    // (and parked) multiple times.
    let _request_span_guard = request_span.enter(); // The span only starts working when we enter here
                                                    // and it will be closed when we drop this guard (similar to C++ RAII)

    let query_span = tracing::info_span!("Saving new subscriber details in the database.");

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
    match insert_query
        .execute(pool.get_ref())
        .instrument(query_span)
        .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok()
        }
        Err(e) => {
            // We use debug formatting here because we want access to more information
            // about the error in the logs.
            tracing::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError()
        }
    }
}
