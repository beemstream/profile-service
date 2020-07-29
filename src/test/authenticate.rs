use super::{get_access_token, run_test, create_user};
use rocket::{http::{Header, ContentType, Status}, local::Client};

#[test]
fn authenticates_token_successfully() {
    run_test(|| {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        create_user(&client, "authenticate");

        let token_response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "authenticate", "password": "Ibrahim123123" }"#)
            .dispatch();

        let access_token = get_access_token(token_response);

        let mut request = client
            .get("/authenticate")
            .header(ContentType::JSON);

        request.add_header(Header::new("token", access_token));

        let response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
    });
}

#[test]
fn does_not_authenticates_token() {
    run_test(|| {
        let client = Client::new(crate::get_rocket()).expect("valid rocket instance");
        create_user(&client, "fail_authenticate");

        let mut request = client
            .get("/authenticate")
            .header(ContentType::JSON);

        request.add_header(Header::new("token", "invalid token"));

        let response = request.dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    });
}

