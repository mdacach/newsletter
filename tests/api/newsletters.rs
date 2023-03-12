use crate::helpers;
use crate::helpers::assert_is_redirect_to;

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

    app.login_with_test_user().await;

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn requests_from_unlogged_users_are_redirected_to_login() {
    let app = helpers::spawn_app().await;

    let body = serde_json::json!({
            "title": "Newsletter title",
            "content": "some content"
    });
    let response = app.post_newsletters(body).await;

    assert_is_redirect_to(&response, "/login");
}
