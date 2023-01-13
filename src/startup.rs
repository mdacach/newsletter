use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};

use crate::routes::health_check;
use crate::routes::subscribe;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            // Note that order here is important, if we had a dynamic /{name} route first,
            // requests to /health_check would match {name}
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run(); // It does not run yet because we have not awaited it

    Ok(server)
}
