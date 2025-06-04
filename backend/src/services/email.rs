#![allow(dead_code)]

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::{MultiPart, SinglePart, Mailbox};
use rand::Rng;
use std::env;
use tracing::{info, error};
use crate::error::{AppError, ErrorType};

pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Result<Self, AppError> {
        let smtp_username = env::var("SMTP_USERNAME")
            .map_err(|_| AppError::new("SMTP_USERNAME not set", ErrorType::Configuration))?;
        let smtp_password = env::var("SMTP_PASSWORD")
            .map_err(|_| AppError::new("SMTP_PASSWORD not set", ErrorType::Configuration))?;
        let smtp_server = env::var("SMTP_SERVER")
            .unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let from_email = env::var("FROM_EMAIL")
            .map_err(|_| AppError::new("FROM_EMAIL not set", ErrorType::Configuration))?;

        info!("Initializing email service with SMTP server: {}", smtp_server);

        let creds = Credentials::new(smtp_username, smtp_password);
        let mailer = SmtpTransport::relay(&smtp_server)
            .map_err(|e| AppError::new(&format!("Failed to create SMTP transport: {}", e), ErrorType::Configuration))?
            .credentials(creds)
            .build();

        info!("Email service initialized successfully");

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

    pub fn send_otp(&self, to_email: &str, otp: &str) -> Result<(), AppError> {
        info!("Sending OTP email to: {}", to_email);

        // Create a mailbox with a friendly name
        let from_mailbox = Mailbox::new(
            Some("LinkSphere".into()),
            self.from_email.parse()
                .map_err(|e| AppError::new(&format!("Invalid from email address: {}", e), ErrorType::Validation))?
        );

        let to_mailbox = to_email.parse()
            .map_err(|e| AppError::new(&format!("Invalid recipient email address: {}", e), ErrorType::Validation))?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject("Your LinkSphere OTP Code")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::plain(format!(
                            "Your verification code is: {}\n\nThis code will expire in 10 minutes.\n\nIf you didn't request this code, please ignore this email.",
                            otp
                        ))
                    )
                    .singlepart(
                        SinglePart::html(format!(
                            r#"
                            <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                                <h1 style="color: #6366f1; text-align: center;">LinkSphere</h1>
                                <div style="background-color: #f3f4f6; border-radius: 8px; padding: 20px; margin-top: 20px;">
                                    <h2 style="color: #374151; margin-bottom: 16px;">Verify Your Email</h2>
                                    <p style="color: #4b5563; margin-bottom: 24px;">Your verification code is:</p>
                                    <div style="background-color: #ffffff; padding: 16px; border-radius: 4px; text-align: center; font-size: 24px; font-weight: bold; letter-spacing: 4px; color: #6366f1;">
                                        {otp}
                                    </div>
                                    <p style="color: #4b5563; margin-top: 24px;">This code will expire in 10 minutes.</p>
                                    <p style="color: #9ca3af; margin-top: 24px; font-size: 14px;">If you didn't request this code, please ignore this email.</p>
                                </div>
                            </div>
                            "#,
                            otp = otp
                        ))
                    )
            )
            .map_err(|e| AppError::new(&format!("Failed to build email: {}", e), ErrorType::Internal))?;

        match self.smtp_transport.send(&email) {
            Ok(_) => {
                info!("Successfully sent OTP email to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send OTP email: {}", e);
                Err(AppError::new(&format!("Failed to send email: {}", e), ErrorType::Internal))
            }
        }
    }
} 