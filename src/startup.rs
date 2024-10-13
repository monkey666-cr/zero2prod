use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use sqlx::{MySql, Pool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub fn run(
    listener: TcpListener,
    connection: Pool<MySql>,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("Hello, world!") }),
            )
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
