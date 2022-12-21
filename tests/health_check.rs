#[tokio::test]
async fn health_check_works() {
    spawn_app();
    // Now the server is running and we can proceed with ou test logic

    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn spawn_app() {
    // This returs a `Server`, which can be awaited (or polled)
    let server = zero2prod::run().expect("Failed to run server");

    let _ = tokio::spawn(server); // We are not doing anything to the handle
}
