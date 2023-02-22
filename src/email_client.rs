use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;
use std::str::FromStr;

use crate::configuration::SMTPSettings;
use crate::domain::SubscriberEmail;
use crate::routes::SendEmailError;

pub struct EmailClient {
    mailer: SmtpTransport,
    from: Mailbox,
}

impl EmailClient {
    pub fn from_settings(settings: &SMTPSettings) -> Self {
        let credentials = Credentials::new(
            settings.username.clone(),
            settings.password.expose_secret().clone(),
        );

        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(credentials)
            .build();

        let from_email =
            Mailbox::from_str(settings.from.as_str()).expect("Invalid 'from' in SMTP settings");

        Self {
            mailer,
            from: from_email,
        }
    }

    pub fn send_email(
        &self,
        to: &SubscriberEmail,
        subject: &str,
        body: &str,
    ) -> Result<(), SendEmailError> {
        let test_email = Message::builder()
            .from(self.from.clone())
            .to(to.as_ref().parse().unwrap())
            .subject(subject)
            .body(body.to_string())
            .expect("Could not send email ");

        let sent_email = self.mailer.send(&test_email);
        match sent_email {
            Ok(_) => Ok(()),
            Err(_) => Err(SendEmailError(
                "Could not send email through SMTP".to_string(),
            )),
        }
    }
}
