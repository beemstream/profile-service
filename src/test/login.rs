use super::{create_user, run_test, get_client};
use rocket::http::{Status, ContentType};

#[test]
fn login_user_successfully_with_username() {
    run_test(|| {
        let client = get_client();
        create_user(&client, "login");

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "login", "password": "Ibrahim123123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let body = response.into_string().unwrap();
        assert_eq!(body.contains("\"status\":\"ok\""), true);
        assert_eq!(body.contains("\"access_token\""), true);
        assert_eq!(body.contains("\"expires_in\""), true);
    });
}

#[test]
fn login_user_successfully_with_email() {
    run_test(|| {
        let client = get_client();
        create_user(&client, "email_login");

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "email_login@gmail.com", "password": "Ibrahim123123" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert_eq!(body.contains("\"status\":\"ok\""), true);
        assert_eq!(body.contains("\"access_token\""), true);
        assert_eq!(body.contains("\"expires_in\""), true);
    });
}

#[test]
fn fails_login_username_with_wrong_password() {
    run_test(|| {
        let client = get_client();
        create_user(&client, "wrongpassword");

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "wrongpassword", "password": "invalid_password" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);

        let body = response.into_string().unwrap();

        assert_eq!(body.contains("not ok"), true);
        assert_eq!(body.contains("Username/email or password is incorrect."), true);
    });
}

#[test]
fn fails_login_email_with_wrong_password() {
    run_test(|| {
        let client = get_client();
        create_user(&client, "wrongpasswordemail");

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "wrongpasswordemail", "password": "invalid_password" }"#)
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);

        let body = response.into_string().unwrap();

        assert_eq!(body.contains("not ok"), true);
        assert_eq!(body.contains("Username/email or password is incorrect."), true);
    });
}

