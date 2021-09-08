use crate::api_model::SendEmail;
use crate::command_line_interface::CliOptions;
use crate::constants::PLUGIN_EMAIL_FOOTER;
use crate::constants::PLUGIN_EMAIL_SENDER;
use crate::constants::PLUGIN_EMAIL_SUBJECT_PREFIX;
use crate::error::Result;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;

pub fn send_email(email: SendEmail, cli: &CliOptions) -> Result<()> {
    match (
        &cli.email_smtp_relay,
        cli.email_smtp_port,
        &cli.email_smtp_user,
        &cli.email_smtp_password,
    ) {
        (Some(relay), Some(port), Some(user), Some(password)) => {
            let to: Mailbox = email.to.parse()?;
            let email = Message::builder()
                .from(PLUGIN_EMAIL_SENDER.parse()?)
                .to(to)
                .subject(format!("{}{}", PLUGIN_EMAIL_SUBJECT_PREFIX, email.subject))
                .body(format!("{}{}", PLUGIN_EMAIL_FOOTER, email.body))
                .unwrap();
            let credentials: Credentials = Credentials::new(user.to_string(), password.to_string());
            let server = SmtpTransport::relay(relay)?
                .port(port)
                .credentials(credentials)
                .build();
            server.send(&email)?;
            Ok(())
        }
        _ => {
            debug_email(email);
            Ok(())
        }
    }
}

fn debug_email(email: SendEmail) {
    log::info!("Email server not configured, debugging email instead");
    log::info!("To: {}", email.to);
    log::info!("Subject: {}", email.subject);
    log::info!("{}", email.body);
}
