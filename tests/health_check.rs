//! tests/health_check.rs

use std::net::TcpListener;

// `tokio::test`是`tokio::main`的测试等价物
// 它还使你不必制定`#[test]`属性
//
// 可以使用一下命令检查生成了哪些代码
// `cargo expand --test health_check`
use zero2prod;

#[tokio::test]
async fn health_check_works() {
    // 准备
    let address = spawn_app();
    // 需要引入reqwest对应程序执行HTTP请求
    let client = reqwest::Client::new();

    // 执行
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // 断言
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
