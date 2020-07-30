use super::{create_user, run_test, get_access_token, get_client};
use rocket::http::{Header, ContentType, Status};

#[test]
fn refreshes_token_successfully() {
    run_test(|| {
        let client = get_client();
        create_user(&client, "refrestokenuser");

        let mut token_response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{ "identifier": "refrestokenuser", "password": "Ibrahim123123" }"#)
            .dispatch();

        let mut request = client
            .get("/refresh-token")
            .header(ContentType::JSON);

        let access_token = get_access_token(token_response.body_string());
        request.add_header(Header::new("token", access_token));

        let response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
    });
}

