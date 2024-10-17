use lettre::transport::smtp::response::{Category, Detail, Severity};
use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, response::Code, response::Response, Error},
    Message, SmtpTransport, Transport,
};

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct MockSmtpTransport {}

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

impl MyTransport for MockSmtpTransport {
    fn clone_box(&self) -> Box<dyn MyTransport<Ok = Response, Error = Error> + Send + Sync> {
        Box::new(self.clone())
    }
}

pub trait MyTransport: Transport<Ok = Response, Error = Error> + Send + Sync {
    fn clone_box(&self) -> Box<dyn MyTransport<Ok = Response, Error = Error> + Send + Sync>;
}

impl MyTransport for SmtpTransport {
    fn clone_box(&self) -> Box<dyn MyTransport<Ok = Response, Error = Error> + Send + Sync> {
        Box::new(self.clone())
    }
}

pub struct TransportWrapper {
    pub mailer: Box<dyn MyTransport>,
}

impl TransportWrapper {
    pub fn new(transport: Box<dyn MyTransport>) -> Self {
        TransportWrapper { mailer: transport }
    }
}

impl Clone for TransportWrapper {
    fn clone(&self) -> Self {
        TransportWrapper {
            mailer: self.mailer.clone_box(),
        }
    }
}

#[derive(Clone)]
pub struct EmailClient {
    transport: TransportWrapper,
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

        let transport = TransportWrapper::new(mailer);

        EmailClient { transport, sender }
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

        match self.transport.mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("邮件发送失败, {e:?}")),
        }
    }

    pub fn set_transport(&mut self, transport: Box<dyn MyTransport>) {
        self.transport = TransportWrapper::new(transport);
    }
}

#[cfg(test)]
mod tests {
    use super::{MockSmtpTransport, TransportWrapper};
    use crate::domain::SubscriberEmail;
    use claim::assert_ok;

    #[tokio::test]
    async fn test_email_send() {
        let transport = MockSmtpTransport {};

        let email_client = super::EmailClient {
            transport: TransportWrapper::new(Box::new(transport)),
            sender: SubscriberEmail::parse("test@example.com".to_string()).unwrap(),
        };

        let recipient = SubscriberEmail::parse("123@qq.com".to_string()).unwrap();

        let result = email_client
            .send_email(&recipient, "test", "<h1>hello</h1>", "")
            .await;

        assert_ok!(result);
    }
}
