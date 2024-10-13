use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, response::Response, Error},
    Message, SmtpTransport, Transport,
};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    mailer: Box<dyn Transport<Ok = Response, Error = Error> + Send + Sync>,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, token: String) -> EmailClient {
        let creds = Credentials::new(sender.to_string(), token.to_owned());

        let mailer = SmtpTransport::relay(&base_url)
            .unwrap()
            .credentials(creds)
            .build();

        let mailer = Box::new(mailer);

        EmailClient { mailer, sender }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        _text_content: &str,
    ) -> Result<(), String> {
        let email = Message::builder()
            .from((&self.sender.to_string()).parse().unwrap())
            .to(recipient.to_string().parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(String::from(html_content))
            .unwrap();

        match self.mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("邮件发送失败, {e:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use lettre::transport::smtp::response::{Category, Detail, Severity};
    use lettre::{
        transport::smtp::{
            response::{Code, Response},
            Error,
        },
        Transport,
    };

    use crate::domain::SubscriberEmail;
    use claim::assert_ok;

    struct MockSmtpTransport {}

    impl Transport for MockSmtpTransport {
        type Ok = Response;

        type Error = Error;

        fn send_raw(
            &self,
            _envelope: &lettre::address::Envelope,
            _email: &[u8],
        ) -> Result<Self::Ok, Self::Error> {
            let code = Code::new(
                Severity::PositiveCompletion,
                Category::Information,
                Detail::Zero,
            );

            let response = Response::new(code, vec!["Ok".to_string()]);

            Ok(response)
        }
    }

    #[tokio::test]
    async fn test_email_send() {
        let transport = MockSmtpTransport {};

        let email_client = super::EmailClient {
            mailer: Box::new(transport),
            sender: SubscriberEmail::parse("test@example.com".to_string()).unwrap(),
        };

        let recipient = SubscriberEmail::parse("123@qq.com".to_string()).unwrap();

        let result = email_client
            .send_email(&recipient, "test", "<h1>hello</h1>", "")
            .await;

        assert_ok!(result);
    }
}
