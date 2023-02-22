use newsletter::configuration::get_configuration;
use newsletter::startup::Application;

use newsletter::telemetry;

#[tokio::main]
// Result so that we can propagate the errors with `?`
async fn main() -> std::io::Result<()> {
    // We need our application to be Observable.
    // This subscriber will log relevant information using `tracing`.
    // We can use such information to debug the application, even in production.
    let subscriber = telemetry::get_subscriber(
        "newsletter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    // Parse configuration from two sources:
    // 1. configuration folder with hierarchical yaml files
    // 2. environment variables (good for secrets)
    let configuration = get_configuration().expect("Failed to read configuration.");

    // The application needs to spin up the webserver and the database.
    let application = Application::build(&configuration).await?;
    application.run_until_stopped().await?; // Runs indefinitely, as a server probably should.

    Ok(())
}
