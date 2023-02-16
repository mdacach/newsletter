use crate::helpers;

#[tokio::test]
async fn health_check_works() {
    let app = helpers::spawn_app().await;
    // Now the server is running and we can proceed with our test logic

    // With reqwest, we approach it as a user would, performing requests
    // from outside.
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    // Simple health check -> should not return any content.
    assert_eq!(response.content_length(), Some(0));
}
