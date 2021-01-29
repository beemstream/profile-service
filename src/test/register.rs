use super::get_client;
use rocket::http::{ContentType, Status};

#[test]
fn creates_user_successfully() {
    let client = get_client();
    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "ibrahim2", "email": "ibrahim2@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();
    assert_eq!(response.status(), Status::Created);
}

#[test]
fn cannot_create_with_not_same_password() {
    let client = get_client();
    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "ibrahim", "email": "ibrahim@gmail.com", "password": "Ibrahim123123", "password_repeat": "not_same" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);

    let body = response.into_string().unwrap();

    assert_eq!(body.contains("password_not_matching"), true);
}

#[test]
fn cannot_create_user_with_same_username() {
    let client = get_client();
    client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();

    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
    assert_eq!(
        response.into_string().unwrap().contains("username_exists"),
        true
    );
}

#[test]
fn cannot_create_user_with_same_email() {
    let client = get_client();
    client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "foobar3", "email": "foobar3@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();
    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "different_username", "email": "foobar3@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::Conflict);
    assert_eq!(
        response.into_string().unwrap().contains("email_exists"),
        true
    );
}

#[test]
fn cannot_create_user_with_not_strong_password() {
    let client = get_client();
    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "bazfoo", "email": "bazfoo@gmail.com", "password": "Ibrahim123", "password_repeat": "Ibrahim123" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    assert_eq!(
        response
            .into_string()
            .unwrap()
            .contains("password_length_invalid"),
        true
    );
}

#[test]
fn cannot_create_user_with_incorrect_email() {
    let client = get_client();
    let response = client
        .post("/auth/register")
        .header(ContentType::JSON)
        .body(r#"{ "username": "bazfoo", "email": "invalid_email", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
        .dispatch();

    assert_eq!(response.status(), Status::UnprocessableEntity);
    let body = response.into_string().unwrap();
    assert_eq!(body.contains("email_invalid"), true);
}
