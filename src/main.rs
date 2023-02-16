use newsletter::configuration::get_configuration;

use newsletter::startup::build;
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
    let server = build(configuration).await?;
    server.await?;
    Ok(())
}
