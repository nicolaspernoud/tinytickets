use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header, Message, SmtpTransport, Transport};
use std::env;

pub fn send_mail(subject: String, body: String) {
    let server = env::var("MAIL_SERVER").unwrap_or_default();
    let user = env::var("MAIL_USER").unwrap_or_default();
    let password = env::var("MAIL_PASSWORD").unwrap_or_default();
    let to = env::var("MAIL_TO").unwrap_or_default();
    let from = env::var("MAIL_FROM").unwrap_or_default();

    /*let mut mailboxes = Mailboxes::new();
    let addresses = to.split(",");

    for a in addresses {
        let address = a.parse::<Address>().unwrap();
        mailboxes.push(Mailbox::new(None, address));
    }

    let email = Message::builder()
        .from(from.parse().unwrap())
        .reply_to(from.parse().unwrap())
        .header(header::To(mailboxes))
        .subject(subject)
        .body(body)
        .expect("Could not send email : could not create the message.");*/

    let email = Message::builder()
        .from(from.parse().unwrap())
        .reply_to(from.parse().unwrap())
        .to(to.parse().unwrap())
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
