use once_cell::sync;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use newsletter::configuration::{get_configuration, DatabaseSettings};

use newsletter::startup::get_connection_pool;
use newsletter::startup::Application;
use newsletter::telemetry;

// We should only register a subscriber once.
// If we simply have this inside each test, it would be called multiple times.
// Using `once_cell` solves this issue.
static TRACING: sync::Lazy<()> = sync::Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    // This subscriber will keep logging stuff. If we are just running routine tests, this is
    // unnecessary. So by default, if the flag is not set, we will sink the logs ("drop" them).
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    // Performs a post request to subscriptions.
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            // x-www-form-urlencoded is a good way to encode information from Forms.
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request to subscriptions endpoint.")
    }
}

pub async fn spawn_app() -> TestApp {
    // Due to `once_cell`, this only runs once, even if we call it multiple times.
    sync::Lazy::force(&TRACING);

    // The configuration for tests has two details:
    // 1. The database name is randomized. This way we create "temporary" databases, for testing only.
    //    -> This is good for test isolation.
    // 2. The port is set to 0. This way the OS will set a random port.
    //    -> This is good for running multiple tests at the same time.
    let configuration = {
        let mut configuration = get_configuration().expect("Failed to read configuration.");

        // As we need test isolation between the tests, we are going to create a new logical database
        // for each test. This way, tests won't interfere with each other, as each one will use a different
        // database.
        configuration.database.database_name = Uuid::new_v4().to_string(); // Random name for this database

        configuration.application.port = 0; // 0 means a random port.

        configuration
    };

    // Note that it is not necessary to use the `connection_pool` returned here, as we have a
    // specific function for that: `get_connection_pool`.
    configure_database(&configuration.database).await;

    let application = Application::build(&configuration)
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());

    // This makes the server run in another thread.
    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        port: application_port,
        address,
        db_pool: get_connection_pool(&configuration.database),
    };
    add_test_user(&test_app.db_pool).await;

    test_app
}

async fn add_test_user(pool: &PgPool) {
    sqlx::query!(
        "INSERT INTO users (user_id, username, password)\
                  VALUES ($1, $2, $3)",
        Uuid::new_v4(),
        Uuid::new_v4().to_string(),
        Uuid::new_v4().to_string()
    )
    .execute(pool)
    .await
    .expect("Failed to create test users.");
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // By using `connection_string_without_db`, we connect to postgres directly, and can create
    // a new database below.
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");

    // Create a randomized database.
    // As each test calls `configure_database` (because it is inside `spawn_app`), each test will
    // have a unique database to query.
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // We have just created a database with this same name, so `connection_string` is sufficient now.
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    // Our database is a clean slate right now.
    // Migrate it so that it has the same tables as our production one.
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
