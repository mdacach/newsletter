use std::net::TcpListener;

use once_cell::sync;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter::configuration::{get_configuration, DatabaseSettings};
use newsletter::startup::run;
use newsletter::telemetry;

// This should only run one time, not once for each test
// So we wrap it within `once_cell`
static TRACING: sync::Lazy<()> = sync::Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // We have the option of printing the logs when testing too
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        // By default we will just ignore them
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // Runs only if it's the first time
    sync::Lazy::force(&TRACING);

    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // As we need test isolation between the tests, we are going to create a new logical database
    // for each test. This way, tests won't interfere with each other, as each one will use a different
    // database.
    configuration.database.database_name = Uuid::new_v4().to_string(); // Random name for this database

    let connection_pool = configure_database(&configuration.database).await;

    // This returns a `Server`, which can be awaited (or polled)
    let server = run(listener, connection_pool.clone()).expect("Failed to run server");

    let _ = tokio::spawn(server); // We are not doing anything to the handle

    // Return the port so that our tests knows where to request
    // And the pool handle so that they can access the connections
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Establish the connection to Postgres
    // Note that here we do not use the database name, as we want to connect to Postgres directly
    // But the database exists, and it is either
    // 1 - our app database name
    // 2 - a random name for testing purposes only
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
    // Create the initial database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database with file we have saved
    // This will create our needed table
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}