use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::{MySql, Pool};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<Pool<MySql>>) -> impl Responder {
    match sqlx::query!(
        r#"
    INSERT INTO subscriptions (email, name, subscribed_at) VALUES ( ?, ?, ? )"#,
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(), // 200 OK
        Err(e) => {
            println!("Failed to execute query: {}", e);
            // 500 Internal Server Error
            HttpResponse::InternalServerError().finish()
        }
    }
}
