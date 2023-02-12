use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;
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
    // Fly app will have the DATABASE_URL environment variable automatically set here,
    // as we have attached the app to the database.
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        configuration
            .database
            .connection_string()
            .expose_secret()
            .clone()
    });

    tracing::info!("The database URL is: {}", database_url);

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(database_url.as_str())
        .expect("Failed to create Postgres connection pool.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
