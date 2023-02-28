use uuid::Uuid;

use crate::helpers;

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    let app = helpers::spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!( {
                "content": "a simple body"
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "title of the newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = reqwest::Client::new()
            .post(&format!("{}/newsletters", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    let app = helpers::spawn_app().await;

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": "some content"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    let app = helpers::spawn_app().await;

    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!(
            {
                "title": "Newsletter title",
            "content": "some content"
            }
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    )
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    let app = helpers::spawn_app().await;
    let username = &app.test_user.username;
    let random_password = Uuid::new_v4().to_string();
    assert_ne!(app.test_user.password, random_password);

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(random_password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": "some content"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );
}
