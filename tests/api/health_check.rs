use crate::helpers::spawn_app;

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
