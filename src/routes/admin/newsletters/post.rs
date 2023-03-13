use actix_web::web::ReqData;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::idempotency::{save_response, try_processing, IdempotencyKey, NextAction};
use crate::utils::{opaque_error_400, opaque_error_500, see_other};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    content: String,
    idempotency_key: String,
}

#[tracing::instrument(
name = "Publish a newsletter issue",
skip(pool, email_client, form),
fields(user_id = % * user_id)
)]
pub async fn publish_newsletter(
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    form: web::Form<FormData>,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(opaque_error_400)?;
    let transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(opaque_error_500)?
    {
        NextAction::StartProcessing(transaction) => transaction,
        NextAction::ReturnSavedResponse(saved_response) => {
            FlashMessage::info("The newsletter issue has been published!").send();
            return Ok(saved_response);
        }
    };

    let subscribers = get_confirmed_subscribers(&pool)
        .await
        .map_err(opaque_error_500)?;
    for subscriber in subscribers {
        email_client
            .send_email(&subscriber.email, &title, &content)
            .with_context(|| {
                format!(
                    "Failed to send newsletter issue to {}",
                    subscriber.email.as_ref()
                )
            })
            .map_err(opaque_error_500)?;
    }

    FlashMessage::info("The newsletter issue has been published!").send();
    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(opaque_error_500)?;
    Ok(response)
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
