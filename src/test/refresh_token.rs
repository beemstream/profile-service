use super::{create_user, get_access_token, get_client};
use rocket::http::{ContentType, Header, Status};
use std::{thread, time};

#[test]
fn refreshes_token_generates_correct_token_successfully() {
    let client = get_client();
    create_user(&client, "refrestokenuser");

    let token_response = client
        .post("/auth/login")
        .header(ContentType::JSON)
        .body(r#"{ "identifier": "refrestokenuser", "password": "Ibrahim123123" }"#)
        .dispatch();

    let mut request = client.get("/auth/refresh-token").header(ContentType::JSON);

    let access_token = get_access_token(&token_response.into_string());
    let header_token = Header::new("token", format!("Bearer {}", access_token));
    request.add_header(header_token);

    let response = request.dispatch();

    assert_eq!(response.status(), Status::Ok);

    let new_access_token = get_access_token(&response.into_string());
    let authenticated_response = client
        .get("/auth/authenticate")
        .header(Header::new("token", format!("Bearer {}", new_access_token)))
        .dispatch();

    assert_eq!(authenticated_response.status(), Status::Ok);
}

#[test]
fn does_not_refreshes_token_with_invalid_token() {
    let client = get_client();
    create_user(&client, "refresh_token_incorrect_token");

    client
        .post("/auth/login")
        .header(ContentType::JSON)
        .body(r#"{ "identifier": "refresh_token_incorrect_token", "password": "Ibrahim123123" }"#)
        .dispatch();

    let mut request = client.get("/auth/refresh-token").header(ContentType::JSON);

    request.add_header(Header::new("token", "incorrect token"));

    let response = request.dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[ignore]
fn system_time_does_not_refresh_token_with_expired_token() {
    let client = get_client();
    create_user(&client, "refresh_token_expired");
    let token_response = client
        .post("/auth/login")
        .header(ContentType::JSON)
        .body(r#"{ "identifier": "refresh_token_expired", "password": "Ibrahim123123" }"#)
        .dispatch();

    let access_token = get_access_token(&token_response.into_string());

    let mut request = client.get("/auth/refresh-token").header(ContentType::JSON);

    request.add_header(Header::new("token", format!("Bearer {}", access_token)));

    thread::sleep(time::Duration::from_millis(4000));
    let response = request.dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}
