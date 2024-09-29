use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::MySqlPool;

use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection =
        MySqlPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to database.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to address");

    run(listener, connection)?.await
}
