//! tests/health_check.rs

use std::net::TcpListener;

use sqlx::{MySql, MySqlPool, Pool};
// `tokio::test`是`tokio::main`的测试等价物
// 它还使你不必制定`#[test]`属性
//
// 可以使用一下命令检查生成了哪些代码
// `cargo expand --test health_check`
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: Pool<MySql>,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = MySqlPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database.");

    let server = run(listener, connection.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection,
    }
}

#[tokio::test]
async fn health_check_works() {
    // 准备
    let app = spawn_app().await;
    // 需要引入reqwest对应程序执行HTTP请求
    let client = reqwest::Client::new();

    // 执行
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // 断言
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=lewis&email=lewis@example.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status());

    let saved =
        sqlx::query!("SELECT email, name FROM subscriptions WHERE email = 'lewis@example.com'",)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "lewis@example.com");
    assert_eq!(saved.name, "lewis");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=lewis", "missing the email"),
        ("email=lewis@example.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status(),
            "The API did not return a 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
