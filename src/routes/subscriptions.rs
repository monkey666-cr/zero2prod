use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::{MySql, Pool};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &Pool<MySql>, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (email, name, subscribed_at) VALUES ( ?, ?, ? )"#,
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscribe_email = %form.email,
        subscribe_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<Pool<MySql>>) -> impl Responder {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
