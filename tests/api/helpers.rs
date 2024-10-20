use std::sync::LazyLock;

use secrecy::ExposeSecret;
use sqlx::{Executor, MySql, MySqlPool, Pool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: Pool<MySql>,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.database.database = format!("test_{}", Uuid::new_v4().as_simple().to_string());
        c.application.port = 0;

        c.email_client.base_url = email_server.uri();

        c
    };

    // 创建并且配置数据库
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),

        db_pool: get_connection_pool(&configuration.database),

        email_server,
    };

    test_app
}

pub async fn configure_database(config: &DatabaseSettings) -> Pool<MySql> {
    let connection = MySqlPool::connect(&config.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to database.");

    connection
        .execute(format!(r#"CREATE DATABASE IF NOT EXISTS {}"#, config.database).as_str())
        .await
        .expect("Failed to create database");

    let connection = MySqlPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&connection)
        .await
        .expect("Failed to migrate the database");

    connection
}
