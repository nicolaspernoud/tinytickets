use lettre::message::header::{self, To};
use lettre::message::{Mailbox, Mailboxes};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, SmtpTransport, Transport};
use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};

pub struct Mailer(pub Arc<Mutex<MailerConfig>>);
impl Clone for Mailer {
    fn clone(&self) -> Self {
        Mailer(self.0.clone())
    }
}

impl Mailer {
    pub fn new(test_mode: bool) -> Self {
        Mailer(Arc::new(Mutex::new(MailerConfig {
            test_mode: test_mode,
            test_mails: HashSet::new(),
        })))
    }

    pub fn send_mail_to(&mut self, subject: String, body: String, to: String) {
        {
            let ref mut this = self.0.lock().unwrap();
            if this.test_mode {
                &this.test_mails.insert(Mail {
                    to: to,
                    subject: subject,
                    body: body,
                });
            } else {
                send_mail_to(subject, body, to);
            };
        };
    }

    pub fn send_mail(&mut self, subject: String, body: String) {
        {
            let to = env::var("MAIL_TO").unwrap_or_default();
            &self.send_mail_to(subject, body, to);
        };
    }

    #[allow(dead_code)]
    pub fn print_test_mails(&self) -> String {
        let mut result: String = "".to_owned();
        for m in &self.0.lock().unwrap().test_mails {
            result.push_str(format!("{:?}", m).as_str())
        }
        result
    }
}

pub struct MailerConfig {
    test_mode: bool,
    test_mails: HashSet<Mail>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Mail {
    to: String,
    subject: String,
    body: String,
}

fn send_mail_to(subject: String, body: String, to: String) {
    let server = env::var("MAIL_SERVER").unwrap_or_default();
    let user = env::var("MAIL_USER").unwrap_or_default();
    let password = env::var("MAIL_PASSWORD").unwrap_or_default();
    let from = env::var("MAIL_FROM").unwrap_or_default();

    let mut mailboxes = Mailboxes::new();
    let addresses = to.split(",");

    for a in addresses {
        let address = a
            .trim()
            .parse::<Address>()
            .expect("The MAIL_TO environnement variable is not set properly.");
        mailboxes.push(Mailbox::new(None, address));
    }

    let email = Message::builder()
        .from(from.parse().unwrap())
        .reply_to(from.parse().unwrap())
        .header(To::from(mailboxes))
        .header(header::ContentType::TEXT_HTML)
        .subject(subject)
        .body(body)
        .expect("Could not send email : could not create the message.");

    let creds = Credentials::new(user, password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&server)
        .expect("Could not send email : could not create the mailer.")
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
