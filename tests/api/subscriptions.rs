use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let body = "name=lewis&email=lewis@example.com";

    let response = app.post_subscriptions(body.into()).await;

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

    let test_cases = vec![
        ("name=lewis", "missing the email"),
        ("email=lewis@example.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status(),
            "The API did not return a 400 Bad Request when the payload was {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_return_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;

    let test_case = vec![
        ("name=&email=lewis@example.com", "empty name"),
        ("name=Ursula&email=", "empty emtail"),
        ("name=Ursula&email=definitely-no-an-email", "invalid emtail"),
    ];

    for (body, message) in test_case {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}",
            message
        );
    }
}
