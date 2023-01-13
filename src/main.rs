use std::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("localhost:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}
