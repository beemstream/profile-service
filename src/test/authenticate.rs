use super::{create_user, get_access_token, get_client};
use rocket::http::{ContentType, Header, Status};
use std::{thread, time};

#[test]
fn authenticates_token_successfully() {
    let client = get_client();
    create_user(&client, "authenticate");

    let token_response = client
        .post("/auth/login")
        .header(ContentType::JSON)
        .body(r#"{ "identifier": "authenticate", "password": "Ibrahim123123" }"#)
        .dispatch();

    let access_token = get_access_token(&token_response.into_string());

    let mut request = client.get("/auth/authenticate").header(ContentType::JSON);

    request.add_header(Header::new("token", format!("Bearer {}", access_token)));

    let response = request.dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn does_not_authenticates_token() {
    let client = get_client();
    create_user(&client, "fail_authenticate");

    let mut request = client.get("/auth/authenticate").header(ContentType::JSON);

    request.add_header(Header::new("token", "invalid token"));

    let response = request.dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[ignore]
fn system_time_does_not_authenticate_token_when_expired() {
    let client = get_client();
    create_user(&client, "expire_auth");

    let token_response = client
        .post("/auth/login")
        .header(ContentType::JSON)
        .body(r#"{ "identifier": "expire_auth", "password": "Ibrahim123123" }"#)
        .dispatch();

    let token = get_access_token(&token_response.into_string());
    let mut request = client.get("/auth/authenticate").header(ContentType::JSON);

    request.add_header(Header::new("token", format!("Bearer {}", token)));

    thread::sleep(time::Duration::from_millis(4000));
    let response = request.dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}
