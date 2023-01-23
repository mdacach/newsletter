use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber =
        telemetry::get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("localhost:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
