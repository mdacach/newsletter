use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::subscribe;
use crate::routes::{confirm, health_check};

// We need a way to know which port the application is running,
// and Server alone does not expose that.
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
        // Get the connection pool from already-running database.
        // (The docker script spins up postgres)
        let connection_pool = get_connection_pool(&configuration.database);

        // We use SMTP to send emails. The credentials are set inside EmailClient.
        // From here on, we can just use it directly.
        let email_client = EmailClient::from_settings(&configuration.smtp);

        // Address we are going to use for our application.
        // This address may change between environments.
        // 1. For local environment, we want to use
        //    127.0.0.1 to only accept local connections.
        // 2. For production environment, we should use 0.0.0.0 so that it can receive
        //    connections from anyone.
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        // If we are in production, the base_url is already everything needed to access our application
        // but in local, the base_url is missing the port, so we add it manually here (for testing purposes).
        let base_url = {
            let mut url = configuration.application.base_url.clone();
            let environment = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());
            if environment == "local" {
                url = format!("{}:{}", url, port);
            }
            url
        };
        let server = run(listener, connection_pool, email_client, base_url)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

// We need to wrap it into a type in order to be able to extract it with actix-web.
#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

// A connection pool is a good approach for when we want to have multiple users using the same database.
// An alternative would be to wrap the PgConnection in a Mutex, but this will have efficiency problems,
// as each user would need to wait for the lock.
// With a PgPool, we let Postgres handle the concurrency.
pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    // The database exposed by `configuration.connection_string()` is already running,
    // because of the Docker script. Here we just need to connect to it.

    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.connection_string().expose_secret())
        .expect("Wrong Database URL format.")
}

fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    // First create the shareable state, and then move inside the closure
    // otherwise you would create it multiple times, every time the closure
    // runs.
    // web::Data is an ARC, so we can clone it inside the closure
    let db_pool = web::Data::new(db_pool);

    let email_client = web::Data::new(email_client);

    let base_url = web::Data::new(ApplicationBaseUrl(base_url));

    // HttpServer receives a closure returning an App
    // It will call this closure in multiple threads (to create a multi-threaded
    // web server).
    // This means anything inside the closure (the connection in this case), must be
    // shareable between threads, which is not the case of PgConnection (as it sits on
    // top of a TCP connection itself), thus we use a PgPool instead.
    let server = HttpServer::new(move || {
        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            // Note that order here is important, if we had a dynamic /{name} route first,
            // requests to /health_check would match {name}
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            // Shareable state between handlers
            .app_data(db_pool.clone()) // Here we pass a clone
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run(); // It does not run yet because we have not awaited it

    Ok(server)
}
