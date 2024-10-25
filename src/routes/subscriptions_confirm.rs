use actix_web::{web, HttpResponse};
use sqlx::{MySql, Pool};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    // 根据token获取订阅ID, 修改订阅ID的状态为已确认
    let subscription_token = &parameters.subscription_token;

    let subscription_id = match get_subscriber_id_from_token(&pool, subscription_token).await {
        Ok(subscription_id) => subscription_id,
        Err(e) => {
            tracing::error!("Failed to get subscription id: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    if confirm_subscriber(&pool, subscription_id).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool))]
pub async fn confirm_subscriber(pool: &Pool<MySql>, subscriber_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = ?"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber id from token", skip(pool))]
pub async fn get_subscriber_id_from_token(
    pool: &Pool<MySql>,
    subscription_token: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscription_id FROM subscription_tokens WHERE token = ?"#,
        subscription_token,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    });

    result.map(|r| r.subscription_id as u64)
}
