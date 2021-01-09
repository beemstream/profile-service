use std::time::Duration;

use lettre::{
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        client::{Tls, TlsParameters},
        PoolConfig,
    },
    AsyncSmtpTransport, Message, SmtpTransport, Tokio02Connector, Tokio02Transport, Transport,
};

pub async fn send_email(to: &str) {
    let email = Message::builder()
        .from(
            "Ibrahim Mahmood <ibrahimpmahmood@gmail.com>"
                .parse()
                .unwrap(),
        )
        .to("Ibrahim Mahmood <ibrahimpictureone@gmail.com>"
            .parse()
            .unwrap())
        .subject("Happy new year")
        .body("Be happy!")
        .unwrap();

    let async_mailer: AsyncSmtpTransport<Tokio02Connector> =
        AsyncSmtpTransport::<Tokio02Connector>::starttls_relay("smtp.gmail.com")
            .unwrap()
            .credentials(Credentials::new(
                "ibrahimpmahmood@gmail.com".to_owned(),
                "xuxncuoalekycnua".to_owned(),
            ))
            .authentication(vec![Mechanism::Plain])
            .build();

    match async_mailer.send(email).await {
        Ok(_) => println!("Email sent"),
        Err(e) => println!("Failed to send email {:?}", e),
    }
}
