use actix_web::web::ReqData;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::utils::{opaque_error_500, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    content: String,
    idempotency_key: String,
}

#[tracing::instrument(
name = "Publish a newsletter issue",
skip(pool, email_client, form),
fields(user_id = % * _user_id)
)]
pub async fn publish_newsletter(
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    form: web::Form<FormData>,
    _user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscribers(&pool)
        .await
        .map_err(opaque_error_500)?;

    for subscriber in subscribers {
        email_client
            .send_email(&subscriber.email, &form.0.title, &form.0.content)
            .with_context(|| {
                format!(
                    "Failed to send newsletter issue to {}",
                    subscriber.email.as_ref()
                )
            })
            .map_err(opaque_error_500)?;
    }

    FlashMessage::info("The newsletter issue has been published!").send();
    Ok(see_other("/admin/newsletters"))
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .filter_map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Some(ConfirmedSubscriber { email }),
            Err(error) => {
                tracing::warn!(
                    // We warn the operator that some email is wrong, so that they can fix it.
                    // But otherwise, we proceed with the newsletter deliver.
                    "A confirmed subscriber is using an invalid email address.\n{}",
                    error
                );
                None
            }
        })
        .collect();

    Ok(confirmed_subscribers)
}
