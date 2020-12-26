use super::{get_client, run_test};
use rocket::http::{ContentType, Status};

#[test]
fn creates_user_successfully() {
    run_test(|| {
        let client = get_client();
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "ibrahim", "email": "ibrahim@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("{\"status\":\"ok\"}".into()));
    });
}

#[test]
fn cannot_create_with_not_same_password() {
    run_test(|| {
        let client = get_client();
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "ibrahim", "email": "ibrahim@gmail.com", "password": "Ibrahim123123", "password_repeat": "not_same" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);

        let body = response.into_string().unwrap();

        assert_eq!(body.contains("not ok"), true);
        assert_eq!(body.contains("Password does not match."), true);
    });
}

#[test]
fn cannot_create_user_with_same_username() {
    run_test(|| {
        let client = get_client();
        client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();

        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "foobar", "email": "foobar@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(
            response
                .into_string()
                .unwrap()
                .contains("Username already exists."),
            true
        );
    });
}

#[test]
fn cannot_create_user_with_same_email() {
    run_test(|| {
        let client = get_client();
        client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "foobar3", "email": "foobar3@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "different_username", "email": "foobar3@gmail.com", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(
            response
                .into_string()
                .unwrap()
                .contains("Email already exists."),
            true
        );
    });
}

#[test]
fn cannot_create_user_with_not_strong_password() {
    run_test(|| {
        let client = get_client();
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "bazfoo", "email": "bazfoo@gmail.com", "password": "Ibrahim123", "password_repeat": "Ibrahim123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(
            response
                .into_string()
                .unwrap()
                .contains("Password must be 12 characters or more"),
            true
        );
    });
}

#[test]
fn cannot_create_user_with_incorrect_email() {
    run_test(|| {
        let client = get_client();
        let response = client
            .post("/register")
            .header(ContentType::JSON)
            .body(r#"{ "username": "bazfoo", "email": "invalid_email", "password": "Ibrahim123123", "password_repeat": "Ibrahim123123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let body = response.into_string().unwrap();
        assert_eq!(body.contains("not ok"), true);
        assert_eq!(body.contains("Please enter a valid email address."), true);
    });
}
