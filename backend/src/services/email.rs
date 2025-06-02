use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use std::env;

pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let smtp_username = env::var("SMTP_USERNAME")?;
        let smtp_password = env::var("SMTP_PASSWORD")?;
        let smtp_server = env::var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let from_email = env::var("FROM_EMAIL")?;

        let creds = Credentials::new(smtp_username, smtp_password);
        let mailer = SmtpTransport::relay(&smtp_server)?
            .credentials(creds)
            .build();

        Ok(Self {
            smtp_transport: mailer,
            from_email,
        })
    }

    pub fn generate_otp() -> String {
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    pub fn send_otp(&self, to_email: &str, otp: &str) -> Result<(), Box<dyn std::error::Error>> {
        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .subject("Your LinkSphere OTP Code")
            .body(format!(
                "Your verification code is: {}\n\nThis code will expire in 10 minutes.\n\nIf you didn't request this code, please ignore this email.",
                otp
            ))?;

        self.smtp_transport.send(&email)?;
        Ok(())
    }
} 