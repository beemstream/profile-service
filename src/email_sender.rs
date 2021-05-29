use lettre::{
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, Message, Tokio02Connector, Tokio02Transport,
};

pub async fn send_email(to: String, email_username: String, email_password: String) {
    let email = Message::builder()
        .from(
            "Ibrahim Mahmood <ibrahimpmahmood@gmail.com>"
                .parse()
                .unwrap(),
        )
        .to(to.parse().unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    let async_mailer: AsyncSmtpTransport<Tokio02Connector> =
        AsyncSmtpTransport::<Tokio02Connector>::starttls_relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(email_username, email_password))
            .authentication(vec![Mechanism::Plain])
            .build();

    match async_mailer.send(email).await {
        Ok(_) => println!("Email sent"),
        Err(e) => println!("Failed to send email {:?}", e),
    }
}
