use newsletter::configuration::get_configuration;
use newsletter::startup::Application;

use newsletter::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber(
        "newsletter".to_string(),
        "info".to_string(),
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(&configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
