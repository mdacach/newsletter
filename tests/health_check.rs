use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    // Now the server is running and we can proceed with ou test logic

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn spawn_app() -> String {
    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    // This returns a `Server`, which can be awaited (or polled)
    let server = zero2prod::run(listener).expect("Failed to run server");

    let _ = tokio::spawn(server); // We are not doing anything to the handle

    // Return the port so that our test knows where to request
    format!("http://127.0.0.1:{}", port)
}
