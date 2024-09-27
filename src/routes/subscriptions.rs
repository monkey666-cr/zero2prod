use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::{MySql, Pool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<Pool<MySql>>) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database.");

    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (email, name, subscribed_at) VALUES ( ?, ?, ? )"#,
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            // 200 OK
            tracing::info!("Saved new subscriber details in the database");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            // 500 Internal Server Error
            HttpResponse::InternalServerError().finish()
        }
    }
}
