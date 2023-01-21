use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::health_check;
use crate::routes::subscribe;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // First create the shareable state, and then move inside the closure
    // otherwise you would create it multiple times, every time the closure
    // runs.
    // web::Data is an ARC, so we can clone it inside the closure
    let db_pool = web::Data::new(db_pool);

    // HttpServer receives a closure returning an App
    // It will call this closure in multiple threads (to create a multi-threaded
    // web server).
    // This means anything inside the closure (the connection in this case), must be
    // shareable between threads, which is not the case of PgConnection (as it sits on
    // top of a TCP connection itself).
    let server = HttpServer::new(move || {
        App::new()
            // Middleware
            .wrap(Logger::default())
            // Note that order here is important, if we had a dynamic /{name} route first,
            // requests to /health_check would match {name}
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone()) // Here we pass a clone
    })
    .listen(listener)?
    .run(); // It does not run yet because we have not awaited it

    Ok(server)
}
