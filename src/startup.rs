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
        let connection_pool = get_connection_pool(&configuration.database);

        // This will eventually be used by the other functions.
        let email_client = EmailClient::from_settings(&configuration.smtp);

        // Address we are going to use for our application.
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url.clone(),
        )?;

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

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    // Here the database is already running, by virtue of `configure_database`.
    // So now we want to establish the connection.
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
    // top of a TCP connection itself).
    let server = HttpServer::new(move || {
        App::new()
            // Middleware
            .wrap(TracingLogger::default())
            // Note that order here is important, if we had a dynamic /{name} route first,
            // requests to /health_check would match {name}
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone()) // Here we pass a clone
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run(); // It does not run yet because we have not awaited it

    Ok(server)
}
