use diesel::prelude::*;
use crate::schema::users;
use crate::database::get_pooled_connection;
use std::panic;
use serde_json::{json, Value};
use rocket::{http::{Status, ContentType, Cookie}, local::{Client, LocalResponse}};

mod register;
mod login;
mod authenticate;
mod refresh_token;

lazy_static!{
    pub static ref ROCKET_CLIENT: Client = Client::new(crate::get_rocket()).expect("valid rocket instance");
}

pub fn run_test<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
    {
        // let is_ok = setup().is_ok();

        // if is_ok {
        let result = panic::catch_unwind(|| {
            test()
        });

        assert!(result.is_ok())
            // } else {
            //     println!("Failed setup");
            // }
    }

fn setup() -> std::result::Result<usize, diesel::result::Error> {
    diesel::delete(users::table).execute(&*get_pooled_connection())
}

pub fn get_access_token(body_string: Option<String>) -> String {
    let token: Value = serde_json::from_str(body_string.unwrap().as_str()).unwrap();
    token["access_token"].as_str().unwrap().to_owned()
}

pub fn create_user<'a>(client: &'a Client, username: &str) -> LocalResponse<'a> {
    let json = json!({
        "username": username,
        "email": format!("{}{}", username, "@gmail.com"),
        "password": "Ibrahim123123",
        "password_repeat": "Ibrahim123123"
    });

    let response = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    response
}

pub fn clean_up_user<'a>(client: &'a Client, username: &str) -> LocalResponse<'a> {

    let json = json!({
        "username": username,
        "email": format!("{}{}", username, "@gmail.com"),
        "password": "Ibrahim123123",
        "password_repeat": "Ibrahim123123"
    });

    let response = client
        .post("/register")
        .header(ContentType::JSON)
        .body(json.to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    response
}

pub fn get_client<'a>() -> &'a Client {
    &*ROCKET_CLIENT
}
